use appview_schema::{models::appview::CampsiteInvite, schema::appview::campsite_invite};
use campground_lexicon::gg::campground::campsite::{CampsiteInviteViewBasic, GetCampsiteInvitesOutput};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_invite_view_basic, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[get("/xrpc/gg.campground.membership.getInvites?<campsite_id>&<limit>&<offset>")]
pub async fn get_invites(auth: CampsiteInfo<'_>, campsite_id: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Json<GetCampsiteInvitesOutput>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    if limit < 1 || limit > 100 {
        return Err(XRPCError::BadRequest("Expected limit query to be between and including 1 and 100".to_string()));
    }

    if !has_role_perms_or_owner(auth.campsite, auth.member.clone(), CampsitePermissionConsts::MANAGE_INVITES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    let invites = campsite_invite::table
        .filter(campsite_invite::campsiteid.eq(campsite_id))
        .order_by(
            campsite_invite::createdat.desc()
        )
        .limit(limit)
        .offset(offset)
        .load::<CampsiteInvite>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(campsite_invite_view_basic)
        .collect::<Vec<CampsiteInviteViewBasic>>();

    return Ok(Json(GetCampsiteInvitesOutput { invites }));
}