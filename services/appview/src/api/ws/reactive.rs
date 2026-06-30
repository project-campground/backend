use std::collections::HashMap;

use appview_schema::{models::appview::CampsiteMember, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::permission::PermissionsDictionary;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    WindowExpressionMethods, dsl::count,
};
use lazy_static::lazy_static;
use rsky_common::cbor_to_struct;
use uuid::Uuid;
use ws::Message;

use crate::{
    api::ws::permissions::{aggregate_ws_permissions, update_ws_permissions},
    database::{
        campsites::{get_campsite_member, get_roles_from_db},
        establish_connection,
    },
    helpers::api::handle_all_db_errors,
    helpers::{
        permissions::{
            ContentPermissionConsts,
            fetch::{fetch_all_campsite_permissions, fetch_only_specific_permissions},
            state::PermissionState,
        },
        ws::CampsiteMemberPermissions,
    },
    realtime::{
        data::ReactiveSubjectData,
        frames::{
            SocketDataFrame, SocketFramePermissionViewPayload, SocketFrameSerializer,
            SocketFrameType,
        },
        messages::{SocketInAnyFrame, SocketInFramePayload},
    },
    try_or_continue,
    util::{iter::AggregatePermissions, sql::array_agg},
};

pub enum WebSocketOutput {
    RocketError(String),
    MessagedClose(String, String),
    RocketMessage(Message),
    BinaryData(Vec<u8>),
    Ignore,
    EmptyClose,
}

lazy_static! {
    pub static ref MEMBER_PERMS_DEFAULT: CampsiteMemberPermissions = CampsiteMemberPermissions {
        roles: PermissionsDictionary {
            general: 0,
            content: 0
        },
        bonfires: HashMap::new(),
        categories: HashMap::new(),
        tents: HashMap::new(),
    };
}

/// Handles any type of the message provided from the client's WebSocket or by AppView's Reactive Subject/Observer (see reactive programming).
pub async fn on_rocket_message(
    msg: Message,
    ws_actor: &Option<String>,
    _actor_campsites: &mut Vec<Option<String>>,
    current_campsite: &mut Option<String>,
    current_membership: &mut Option<CampsiteMember>,
    permissions: &mut CampsiteMemberPermissions,
) -> WebSocketOutput {
    match msg {
        ws::Message::Ping(payload) => WebSocketOutput::RocketMessage(ws::Message::Pong(payload)),
        ws::Message::Close(_) => WebSocketOutput::EmptyClose,
        ws::Message::Binary(bytes) => {
            let data = cbor_to_struct::<SocketInAnyFrame>(bytes.clone());

            if data.is_err() {
                // FIXME Used in the debug so far; proper logging is needed in the future
                println!("Data is err: {:?}", data.err().unwrap());

                return WebSocketOutput::MessagedClose(
                    "ParsingError".to_string(),
                    "Could not parse binary CBOR".to_string(),
                );
            }

            let data = data.unwrap();

            // Make sure the first frame is an authentication frame, because non-compliant client could give any other frame to the WS
            match data.op {
                SocketFrameType::Data => {
                    match data.payload {
                        SocketInFramePayload::View(view) => {
                            // FIXME: Proper DEBUG-only logging
                            println!("Change campsite: {:?}", view.campsite.clone());

                            // By default, you can pass the campsite to get events from
                            *current_campsite = if view.campsite == "" {
                                None
                            } else {
                                Some(view.campsite)
                            };

                            if let Some(campsite_id) = current_campsite
                                && let Some(ws_actor) = ws_actor
                            {
                                *current_membership =
                                    get_campsite_member(campsite_id, ws_actor).ok();

                                // Basically empty permissions that are going to be changed anyways (most likely)
                                *permissions = MEMBER_PERMS_DEFAULT.clone();
                                let roles = try_or_continue!(get_roles_from_db(campsite_id).ok());
                                let permissions_list = try_or_continue!(
                                    fetch_all_campsite_permissions(campsite_id, ws_actor).ok()
                                );

                                let aggregated = aggregate_ws_permissions(
                                    &current_membership.clone().unwrap(),
                                    &roles,
                                    &permissions_list,
                                )
                                .await;

                                if let Ok(new_perms) = aggregated {
                                    *permissions = new_perms.clone();
                                };
                            }

                            WebSocketOutput::Ignore
                        }
                        SocketInFramePayload::ViewPermissions => {
                            println!("View permissions");
                            // FIXME This is so far used for the lack of better method and will likely be removed in the future in favour of having client calculate it on its own.
                            // Use references somehow. I hate lack of GC
                            let response = SocketDataFrame::<SocketFramePermissionViewPayload>::new(
                                "PermissionView".to_string(),
                                SocketFramePermissionViewPayload {
                                    permissions: permissions.clone(),
                                },
                            );
                            let binary = response.binary();

                            if let Ok(binary) = binary {
                                WebSocketOutput::BinaryData(binary)
                            } else {
                                WebSocketOutput::RocketError(binary.err().unwrap().to_string())
                            }
                        }
                    }
                }
                _ => WebSocketOutput::RocketError("Unexpected opcode".to_string()),
            }
        }
        _ => WebSocketOutput::Ignore,
    }
}

/// When the WebSocket message is correct, this method properly handles actual events in Campground AppView or WebSocket events.
pub async fn on_reactive_data(
    omsg: ReactiveSubjectData,
    ws_actor: &Option<String>,
    actor_campsites: &mut Vec<Option<String>>,
    current_campsite: &mut Option<String>,
    current_membership: &mut Option<CampsiteMember>,
    permissions: &mut CampsiteMemberPermissions,
) -> WebSocketOutput {
    match omsg {
        ReactiveSubjectData::RocketError => {
            WebSocketOutput::RocketError("WS Error received".to_string())
        }
        // Incoming messages, sent by the user
        ReactiveSubjectData::RocketMessage(msg) => {
            on_rocket_message(
                msg,
                ws_actor,
                actor_campsites,
                current_campsite,
                current_membership,
                permissions,
            )
            .await
        }
        ReactiveSubjectData::MemberRolesModified {
            campsite_id,
            actors,
            role_id,
            removed,
            permissions_are_empty,
            binary,
        } => {
            // Could be used in observer .filter, but borrowing could be less intuitive
            if current_campsite.clone().map_or(true, |current_campsite| {
                current_campsite != campsite_id.clone()
            }) {
                return WebSocketOutput::Ignore;
            // Nothing to modify
            } else if ws_actor
                .clone()
                .map_or(true, |ws_actor| !actors.contains(&ws_actor))
                || current_membership.is_none()
            {
                return WebSocketOutput::BinaryData(binary);
            }

            // Update role list to not refetch membership
            let current_membership_unwrapped = &mut current_membership.clone().unwrap();
            if removed
                && let Some(current_role_index) = current_membership_unwrapped
                    .roles
                    .iter()
                    .position(|x| x.map_or(false, |x| x == role_id))
            {
                current_membership_unwrapped
                    .roles
                    .remove(current_role_index);
            } else {
                current_membership_unwrapped.roles.push(Some(role_id));
            }
            *current_membership = Some(current_membership_unwrapped.clone());

            let mut conn = establish_connection().unwrap();
            // Get what to update in terms of permissions, if anything at all
            let new_permissions_to_update = campsite_permission::table
                .filter(
                    campsite_permission::campsiteid
                        .eq(&campsite_id)
                        .and(campsite_permission::roleid.eq(role_id)),
                )
                .select((
                    // How many to update at all (will be used for bonfire update counting)
                    count(campsite_permission::id),
                    // What bonfires to update
                    array_agg(campsite_permission::bonfireid).window_filter(
                        campsite_permission::categoryid
                            .is_null()
                            .and(campsite_permission::tentid.is_null()),
                    ),
                    // What categories to update
                    array_agg(campsite_permission::categoryid.assume_not_null())
                        .window_filter(campsite_permission::categoryid.is_not_null()),
                    // What tents to update
                    array_agg(campsite_permission::tentid.assume_not_null())
                        .window_filter(campsite_permission::tentid.is_not_null()),
                ))
                .first::<(
                    i64,
                    Option<Vec<String>>,
                    Option<Vec<Uuid>>,
                    Option<Vec<Uuid>>,
                )>(&mut conn)
                .map_err(handle_all_db_errors);

            // No permissions to redo
            if permissions_are_empty
                && new_permissions_to_update
                    .as_ref()
                    .ok()
                    .map_or(true, |x| x.0 == 0)
            {
                return WebSocketOutput::BinaryData(binary);
            }

            let (_, bonfires_to_update, categories_to_update, tents_to_update) =
                new_permissions_to_update.unwrap();

            if let Ok(new_perms) = update_ws_permissions(
                &campsite_id,
                &current_membership_unwrapped,
                permissions,
                permissions_are_empty,
                bonfires_to_update.unwrap_or(vec![]),
                categories_to_update.unwrap_or(vec![]),
                tents_to_update.unwrap_or(vec![]),
            )
            .await
            {
                *permissions = new_perms.clone();
            }

            WebSocketOutput::BinaryData(binary)
        }
        ReactiveSubjectData::CampsitePermissionUpdated {
            campsite_id,
            bonfire_id,
            category_id,
            tent_id,
            user_id,
            role_id,
            binary,
        } => {
            if current_campsite.clone().map_or(true, |current_campsite| current_campsite != campsite_id)
                // Is user that has the permission applied
                || ws_actor.clone().map_or(true, |ws_actor| user_id.map_or(false, |user_id| user_id != ws_actor))
                // Has the role that has the permission applied
                || current_membership.clone().map_or(true, |current_membership| role_id.map_or(false, |role_id| !current_membership.roles.contains(&Some(role_id))))
            {
                return WebSocketOutput::Ignore;
            }

            let ws_actor = ws_actor.clone().unwrap();

            let permissions_result = &fetch_only_specific_permissions(
                &campsite_id,
                &bonfire_id,
                category_id,
                tent_id,
                &ws_actor,
                &current_membership.clone().map_or(vec![], |x| {
                    x.roles
                        .clone()
                        .iter()
                        .filter_map(|&y| y)
                        .collect::<Vec<Uuid>>()
                }),
            )
            .await;
            // Give an error and just disallow putting out events of anything else
            if let Err(err) = permissions_result {
                println!("Error fetching permissions: {:?}", err);
                *permissions = MEMBER_PERMS_DEFAULT.clone();
                return WebSocketOutput::Ignore;
            }
            println!("No error");
            let aggregated = permissions_result
                .as_ref()
                .unwrap()
                .iter()
                .aggregate_permissions();

            // Only update member's permissions in that place, not the whole campsite
            match (category_id, tent_id) {
                (Some(category_id), None) => {
                    permissions.categories.insert(category_id, aggregated);
                }
                (Some(_), Some(tent_id)) | (None, Some(tent_id)) => {
                    permissions.tents.insert(tent_id, aggregated);
                }
                _ => {
                    permissions.bonfires.insert(bonfire_id, aggregated);
                }
            }

            WebSocketOutput::BinaryData(binary)
        }
        // Role created, etc.
        ReactiveSubjectData::Campsite {
            campsite_id,
            permissions_required,
            binary,
        } => {
            if
            // Could be used in observer .filter, but borrowing could be less intuitive
            current_campsite.clone().map_or(true, |current_campsite| current_campsite != campsite_id.clone()) ||
                // For invited created and whatnot
                permissions.roles.general & permissions_required != permissions_required
            {
                return WebSocketOutput::Ignore;
            }

            WebSocketOutput::BinaryData(binary)
        }
        // Bonfire updated, etc.
        ReactiveSubjectData::Bonfire {
            campsite_id,
            bonfire_id,
            binary,
            deleted,
        } => {
            // Could be used in observer .filter, but borrowing could be less intuitive
            if current_campsite.clone().map_or(true, |current_campsite| {
                current_campsite != campsite_id.clone()
            }) {
                return WebSocketOutput::Ignore;
            }

            // Permission check to not have non-mod members see events from mod-only bonfire
            let bonfire_permissions = PermissionState::from_content_optional(
                &permissions.bonfires.get(&bonfire_id),
                ContentPermissionConsts::VIEW_CONTENT,
            )
            .map_inherit(|| {
                PermissionState::from_role_content(
                    &permissions.roles,
                    ContentPermissionConsts::VIEW_CONTENT,
                )
            });

            // To no longer track it
            if deleted {
                permissions.bonfires.remove(&bonfire_id);
            }

            if !bonfire_permissions.is_allowed() {
                return WebSocketOutput::Ignore;
            }

            WebSocketOutput::BinaryData(binary)
        }
        // Category updated, etc.
        ReactiveSubjectData::Category {
            campsite_id,
            bonfire_id,
            category_id,
            binary,
            deleted,
        } => {
            // Could be used in observer .filter, but borrowing could be less intuitive
            if current_campsite.clone().map_or(true, |current_campsite| {
                current_campsite != campsite_id.clone()
            }) {
                return WebSocketOutput::Ignore;
            }

            // Permission check to not have non-mod members see events from mod-only bonfire
            let category_permissions = PermissionState::from_content_optional(
                &permissions.categories.get(&category_id),
                ContentPermissionConsts::VIEW_CONTENT,
            )
            .map_inherit(|| {
                PermissionState::from_content_optional(
                    &permissions.bonfires.get(&bonfire_id),
                    ContentPermissionConsts::VIEW_CONTENT,
                )
                .map_inherit(|| {
                    PermissionState::from_role_content(
                        &permissions.roles,
                        ContentPermissionConsts::VIEW_CONTENT,
                    )
                })
            });
            // To no longer track it
            if deleted {
                permissions.categories.remove(&category_id);
            }

            if !category_permissions.is_allowed() {
                return WebSocketOutput::Ignore;
            }

            WebSocketOutput::BinaryData(binary)
        }
        // Tent updated, message created etc.
        ReactiveSubjectData::Tent {
            campsite_id,
            bonfire_id,
            category_id,
            tent_id,
            binary,
            deleted,
        } => {
            // Could be used in observer .filter, but borrowing could be less intuitive
            if current_campsite.clone().map_or(true, |current_campsite| {
                current_campsite != campsite_id.clone()
            }) {
                return WebSocketOutput::Ignore;
            }

            // Permission check to not have non-mod members see events from mod-only bonfire
            let tent_permissions = PermissionState::from_content_optional(
                &permissions.tents.get(&tent_id),
                ContentPermissionConsts::VIEW_CONTENT,
            )
            .map_inherit(|| {
                PermissionState::from_content_optional(
                    &category_id
                        .map(|x| permissions.categories.get(&x))
                        .flatten(),
                    ContentPermissionConsts::VIEW_CONTENT,
                )
                .map_inherit(|| {
                    PermissionState::from_content_optional(
                        &permissions.bonfires.get(&bonfire_id),
                        ContentPermissionConsts::VIEW_CONTENT,
                    )
                    .map_inherit(|| {
                        PermissionState::from_role_content(
                            &permissions.roles,
                            ContentPermissionConsts::VIEW_CONTENT,
                        )
                    })
                })
            });

            // To no longer track it
            if deleted {
                permissions.tents.remove(&tent_id);
            }

            if !tent_permissions.is_allowed() {
                return WebSocketOutput::Ignore;
            }

            WebSocketOutput::BinaryData(binary)
        }
        // Campsite has been deleted, campsite modified, etc.
        ReactiveSubjectData::CampsiteGlobal {
            campsite_id,
            binary,
        } => {
            if !actor_campsites.contains(&Some(campsite_id.clone())) {
                return WebSocketOutput::Ignore;
            }

            WebSocketOutput::BinaryData(binary)
        }
        // Campsite joined
        ReactiveSubjectData::CampsiteAdded {
            campsite_id,
            to_actor,
            binary,
        } => {
            if ws_actor
                .clone()
                .map_or(true, |ws_actor| to_actor.clone() != ws_actor)
            {
                return WebSocketOutput::Ignore;
            }

            actor_campsites.push(Some(campsite_id));

            WebSocketOutput::BinaryData(binary)
        }
        // Campsite left
        ReactiveSubjectData::CampsiteRemoved {
            campsite_id,
            to_actor,
            binary,
        } => {
            if ws_actor
                .clone()
                .map_or(true, |ws_actor| to_actor.clone() != ws_actor)
            {
                return WebSocketOutput::Ignore;
            } else if current_campsite.clone().map_or(true, |current_campsite| {
                campsite_id.clone() == current_campsite
            }) {
                *current_campsite = None;
                *current_membership = None;
                *permissions = MEMBER_PERMS_DEFAULT.clone();
            }

            // To no longer send global campsite events from
            let campsite_index = actor_campsites
                .iter()
                .position(|x| x.as_ref().map_or(false, |y| *y == campsite_id));
            if campsite_index.is_none() {
                return WebSocketOutput::Ignore;
            }

            actor_campsites.remove(campsite_index.unwrap());

            WebSocketOutput::BinaryData(binary)
        }
        // DM received and whatever
        ReactiveSubjectData::Personal { to_actor, binary } => {
            if ws_actor
                .clone()
                .map_or(true, |ws_actor| to_actor.clone() != ws_actor)
            {
                return WebSocketOutput::Ignore;
            }

            WebSocketOutput::BinaryData(binary)
        }
    }
}
