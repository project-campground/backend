#![allow(unused_variables)]
use appview_schema::{
    models::appview::{Actor, CampsiteMember, Profile, TentMessage},
    schema::appview::{self, campsite_member, profile, tent_message},
};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::message::MessageViewBasic;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
        },
        ws::event_next_tent,
    },
    realtime::data::ReactiveSubject,
    views::messages::message_view_basic,
    xrpc::{
        campsite::TentInfo,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/gg.campground.message.deleteMessage?<tent_id>&<message_id>")]
pub async fn delete_message(
    auth: TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    tent_id: &str,
    message_id: &str,
) -> Result<Json<MessageViewBasic>> {
    let mut conn = establish_connection().unwrap();

    if auth.tent.r#type != 0 {
        return Err(XRPCError::BadRequest(
            "This tent type does not support messages".to_string(),
        ));
    }

    let msg_id_uuid = Uuid::try_parse(message_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))?;
    let msg = &tent_message::table
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(tent_message::tentid.eq(auth.tent.id)),
        )
        .left_join(appview::actor::table.on(tent_message::createdby.eq(appview::actor::did)))
        .left_join(profile::table.on(tent_message::createdby.eq(profile::creator)))
        .left_join(
            campsite_member::table.on(tent_message::createdby
                .eq(campsite_member::userid)
                .and(tent_message::campsiteid.eq(campsite_member::campsiteid))),
        )
        .first::<(
            TentMessage,
            Option<Actor>,
            Option<Profile>,
            Option<CampsiteMember>,
        )>(&mut conn)
        .map_err(handle_select_first_error)?;

    let required_perms = if msg.0.created_by != auth.actor.did {
        PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_CONTENT
    } else {
        PermissionFlag::VIEW_CONTENT
    };

    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| scoped_permission(
                permission.fetch_roles(),
                &auth.campsite.id,
                &auth.tent.bonfire_id,
                auth.tent.category_id.as_ref(),
                Some(&auth.tent.id)
            )
            .has_permissions(required_perms))?
    );

    diesel::delete(tent_message::table)
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(tent_message::tentid.eq(auth.tent.id)),
        )
        .execute(&mut conn)
        .expect("Error deleting message");

    let message = message_view_basic(
        &auth.tent,
        &msg.0,
        msg.1.as_ref(),
        msg.2.as_ref(),
        msg.3.as_ref(),
    );

    event_next_tent(event_subject, &auth.tent, false, "MessageDeleted", &message);

    return Ok(Json(message));
}
