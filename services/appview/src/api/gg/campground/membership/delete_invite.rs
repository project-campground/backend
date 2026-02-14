use appview_schema::{models::appview::CampsiteInvite, schema::appview::campsite_invite};
use campground_lexicon::gg::campground::membership::CampsiteInviteViewBasic;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_invite_view_basic, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[post("/xrpc/gg.campground.membership.deleteInvite?<campsite_id>&<invite_id>")]
pub async fn delete_invite(auth: CampsiteInfo<'_>, campsite_id: &str, invite_id: &str) -> Result<Json<CampsiteInviteViewBasic>> {    
    if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::MANAGE_INVITES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let uuid = Uuid::try_parse(invite_id)
        .map_err(|_| XRPCError::BadRequest("Invalid 'invite_id' format. Expected UUID".to_string()))?;

    let invite = campsite_invite::table
        .filter(campsite_invite::id.eq(uuid))
        .first::<CampsiteInvite>(&mut conn)
        .map_err(handle_select_first_error)?;

    if invite.campsite_id != campsite_id {
        return Err(XRPCError::NotFound);
    }

    return Ok(Json(campsite_invite_view_basic(&invite)));
}