use appview_schema::{models::appview::{CampsiteInvite, CampsiteMember, CampsiteRole}, schema::appview::{self, campsite, campsite_ban, campsite_invite, campsite_member, campsite_role}};
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, PgArrayExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{database::{actors::get_actor, establish_connection}, helpers::{api::handle_select_first_error, roles::CampsiteRoleFlag}, xrpc::{
    auth::Authorization, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.membership.useInvite?<invite_id>")]
pub async fn use_invite(auth: Authorization<'_>, invite_id: &str) -> Result<()> {
    let mut conn = establish_connection().unwrap();
    let actor = &get_actor(auth.client, auth.did_document_storage, auth.actor_did.clone().as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let uuid = Uuid::try_parse(invite_id)
        .map_err(|_| XRPCError::BadRequest("Invalid 'invite_id' format. Expected UUID".to_string()))?;

    let invite = campsite_invite::table
        .filter(campsite_invite::id.eq(uuid))
        .first::<CampsiteInvite>(&mut conn)
        .map_err(handle_select_first_error)?;

    let current_date = Utc::now().naive_utc();

    if actor.campsites.contains(&Some(invite.campsite_id.clone())) {
        return Err(XRPCError::Forbidden("User already joined this campsite".to_string()));
    } else if invite.allowed_amount.map_or(false, |x| x <= invite.used) {
        return Err(XRPCError::Forbidden("This invite has already hit the allowed use limit".to_string()));
    } else if invite.expires_at.map_or(false, |x| x >= current_date) {
        return Err(XRPCError::Forbidden("This invite has already expired".to_string()));
    }
    
    let ban_count = campsite_ban::table
        .filter(campsite_ban::userid.eq(auth.actor_did.clone()).and(campsite_ban::campsiteid.eq(invite.campsite_id.clone())))
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    if ban_count > 0 {
        return Err(XRPCError::Forbidden("User is banned from this campsite".to_string()));
    }

    let roles = campsite_role::table
        .filter(
            campsite_role::campsiteid
                .eq(invite.campsite_id.clone())
        )
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let auto_roles = roles
        .iter()
        .filter(|x| (x.flags & CampsiteRoleFlag::DEFAULT_ROLE) != 0)
        .map(|x| Some(x.id))
        .collect::<Vec<Option<Uuid>>>();

    diesel::update(campsite::table)
        .filter(
            campsite::id
                .eq(invite.campsite_id.clone())
        )
        .set(
            campsite::memberdids
                .eq(
                    campsite::memberdids
                        .concat(vec![auth.actor_did.clone()])
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::update(campsite_invite::table)
        .filter(
            campsite_invite::id
                .eq(&invite.id)
        )
        .set(
            campsite_invite::used
                .eq(
                    invite.used + 1
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::update(appview::actor::table)
        .filter(
            appview::actor::did
                .eq(auth.actor_did.clone())
        )
        .set(
            appview::actor::campsites
                .eq(
                    appview::actor::campsites
                        .concat(vec![invite.campsite_id.clone()])
                )
        )
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    diesel::insert_into(campsite_member::table)
        .values(
            CampsiteMember {
                user_id: auth.actor_did,
                campsite_id: invite.campsite_id,
                joined_at: current_date,
                used_invite_id: Some(uuid),
                nickname: None,
                roles: auto_roles,
            }
        )
        .load::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(());
}