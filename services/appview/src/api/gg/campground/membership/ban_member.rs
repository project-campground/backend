use appview_schema::{models::appview::{Actor, CampsiteBan, CampsiteMember, CampsiteRole, Profile}, schema::appview::{self, campsite_member, campsite_role, profile}};
use campground_lexicon::gg::campground::membership::CampsiteBanView;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{api::gg::campground::membership::remove_member::{ensure_user_isnt_higher, remove_campsite_member}, database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_ban_view, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}, ws::event_next}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateBanBody {
    reason: Option<String>,
}

#[post("/xrpc/gg.campground.membership.banMember?<campsite_id>&<actor>", data = "<body>")]
pub async fn ban_member(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, actor: &str, body: Json<CreateBanBody>) -> Result<Json<CampsiteBanView>> {    
    if actor == auth.actor.did {
        return Err(XRPCError::Forbidden("Member cannot ban themselves".to_string()));
    }

    let inner_body = &body.into_inner();
    if inner_body.reason.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    } else if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::BAN_MEMBERS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    // Make sure the role exists
    let all_roles = campsite_role::table
        .filter(campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    let target = campsite_member::table
        .filter(
            campsite_member::campsiteid
                .eq(campsite_id)
                .and(
                    campsite_member::userid.eq(&auth.actor.did)
                )
        )
        .inner_join(
            profile::table
                .on(
                    profile::creator.eq(
                        campsite_member::userid
                    )
                )
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(
                    crate::schema::appview::actor::did.eq(
                        campsite_member::userid
                    )
                )
        )
        .first::<(CampsiteMember, Profile, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    ensure_user_isnt_higher(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), &target.0.roles, auth.member.roles.clone())?;
    
    remove_campsite_member(event_subject, &auth.campsite.id, &target, actor)?;

    let current_date = Utc::now().naive_utc();
    let ban = &diesel::insert_into(appview::campsite_ban::table)
        .values(
            CampsiteBan {
                user_id: actor.to_string(),
                campsite_id: campsite_id.to_string(),
                reason: inner_body.reason.clone(),
                created_at: current_date,
                created_by: auth.actor.did.clone(),
                updated_at: current_date,
                updated_by: auth.actor.did,
            }
        )
        .load::<CampsiteBan>(&mut conn)
        .map_err(handle_select_first_error)?;

    let ban = ban.first().unwrap();

    event_next(event_subject, &auth.campsite.id, "MemberBanCreated", campsite_ban_view(ban, &target.1, &target.2));
 
    Ok(Json(campsite_ban_view(ban, &target.1, &target.2)))
}
