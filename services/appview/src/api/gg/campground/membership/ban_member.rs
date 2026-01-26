use appview_schema::{models::appview::{CampsiteBan, CampsiteMember, CampsiteRole}, schema::appview::{self, campsite_member, campsite_role}};
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::{api::gg::campground::membership::remove_member::{ensure_user_isnt_higher, remove_campsite_member}, database::establish_connection, helpers::{api::handle_select_first_error, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateBanBody {
    reason: Option<String>,
}

#[post("/xrpc/gg.campground.membership.banMember?<campsite_id>&<actor>", data = "<body>")]
pub async fn ban_member(auth: CampsiteInfo<'_>, campsite_id: &str, actor: &str, body: Json<CreateBanBody>) -> Result<()> {    
    if actor == auth.actor.did {
        return Err(XRPCError::Forbidden("Member cannot ban themselves".to_string()));
    }

    let inner_body = &body.into_inner();
    if inner_body.reason.clone().map_or(false, |x| x.len() > 200) {
        
    }
    else if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::BAN_MEMBERS, 0).await? {
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
        .first::<CampsiteMember>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    ensure_user_isnt_higher(auth.campsite.owner == auth.actor.did, &mut all_roles.clone(), &target.roles, auth.member.roles.clone())?;
    
    remove_campsite_member(campsite_id, actor)?;

    let current_date = Utc::now().naive_utc();
    diesel::insert_into(appview::campsite_ban::table)
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
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;
    
    Ok(())
}
