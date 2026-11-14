use campground_lexicon::chat::campground::campsite::CampsiteViewBasic;
use cds_schema::{
    models::cds::{Campsite, CampsiteInvite, CampsiteMember, Role},
    schema::cds::{self, campsite, campsite_invite, campsite_member, member_ban, role},
};
use chrono::Utc;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, JoinOnDsl, PgArrayExpressionMethods,
    QueryDsl, RunQueryDsl,
};
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::{establish_connection, profiles::get_profile},
    helpers::{
        api::{handle_all_db_errors, handle_select_first_error},
        ws::{event_next, event_next_campsite},
    },
    realtime::data::{ReactiveSubject, ReactiveSubjectData},
    views::{campsites::campsite_view_basic, members::member_view_basic, roles::CampsiteRoleFlag},
    xrpc::{
        auth::Authorization,
        error::{Result, XRPCError},
    },
};

#[post("/xrpc/chat.campground.invite.useInvite?<invite_id>")]
pub async fn use_invite(
    auth: Authorization<'_>,
    event_subject: &State<ReactiveSubject>,
    invite_id: &str,
) -> Result<Json<CampsiteViewBasic>> {
    let mut conn = establish_connection().unwrap();
    let (actor, profile) = &get_profile(auth.client, auth.did_document_storage, &auth.actor_did)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let uuid = Uuid::try_parse(invite_id).map_err(|_| {
        XRPCError::BadRequest("Invalid 'invite_id' format. Expected UUID".to_string())
    })?;

    let (invite, campsite) = campsite_invite::table
        .filter(campsite_invite::id.eq(uuid))
        .inner_join(cds::campsite::table.on(cds::campsite::id.eq(campsite_invite::campsite)))
        .first::<(CampsiteInvite, Campsite)>(&mut conn)
        .map_err(handle_select_first_error)?;

    let current_date = Utc::now().naive_utc();

    if actor.campsites.contains(&Some(invite.campsite.clone())) {
        return Err(XRPCError::Forbidden(
            "User already joined this campsite".to_string(),
        ));
    } else if invite.allowed_uses.map_or(false, |x| x <= invite.used) {
        return Err(XRPCError::Forbidden(
            "This invite has already hit the allowed use limit".to_string(),
        ));
    } else if invite.expires_at.map_or(false, |x| x >= current_date) {
        return Err(XRPCError::Forbidden(
            "This invite has already expired".to_string(),
        ));
    }

    let ban_count = member_ban::table
        .filter(
            member_ban::target
                .eq(auth.actor_did.clone())
                .and(member_ban::campsite.eq(invite.campsite.clone())),
        )
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;

    if ban_count > 0 {
        return Err(XRPCError::Forbidden(
            "User is banned from this campsite".to_string(),
        ));
    }

    let roles = role::table
        .filter(role::campsite.eq(invite.campsite.clone()))
        .load::<Role>(&mut conn)
        .map_err(handle_select_first_error)?;

    let auto_roles = roles
        .iter()
        .filter(|x| (x.flags & CampsiteRoleFlag::DEFAULT_ROLE) != 0)
        .map(|x| Some(x.id))
        .collect::<Vec<Option<Uuid>>>();

    let members = conn
        .transaction(|conn| {
            diesel::update(campsite::table)
                .filter(campsite::id.eq(invite.campsite.clone()))
                .set(campsite::members.eq(campsite::members.concat(vec![auth.actor_did.clone()])))
                .execute(conn)?;

            diesel::update(campsite_invite::table)
                .filter(campsite_invite::id.eq(&invite.id))
                .set(campsite_invite::used.eq(invite.used + 1))
                .execute(conn)?;

            diesel::update(cds::actor::table)
                .filter(cds::actor::did.eq(auth.actor_did.clone()))
                .set(
                    cds::actor::campsites
                        .eq(cds::actor::campsites.concat(vec![invite.campsite.clone()])),
                )
                .execute(conn)?;

            diesel::insert_into(campsite_member::table)
                .values(CampsiteMember {
                    actor: auth.actor_did,
                    campsite: invite.campsite,
                    joined_at: current_date,
                    used_invite: Some(uuid),
                    nickname: None,
                    roles: auto_roles,
                })
                .load::<CampsiteMember>(conn)
        })
        .map_err(handle_all_db_errors)?;

    let member = members.first().unwrap();
    let member_view = member_view_basic(member, profile.as_ref(), actor);
    let campsite_view = campsite_view_basic(&campsite);

    event_next_campsite(event_subject, &campsite.id, 0, "MemberJoined", &member_view);
    event_next(event_subject, "CampsiteJoined", &campsite_view, |binary| {
        ReactiveSubjectData::CampsiteAdded {
            campsite_id: campsite.id.clone(),
            to_actor: actor.did.clone(),
            binary,
        }
    });

    return Ok(Json(campsite_view));
}
