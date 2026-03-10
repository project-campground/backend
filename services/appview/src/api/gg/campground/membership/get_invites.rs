use appview_schema::{models::appview::{Actor, CampsiteInvite, Profile}, schema::appview::{campsite_invite, profile}};
use campground_lexicon::gg::campground::membership::{CampsiteInviteViewCampsite, GetCampsiteInvitesOutput};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_invite_view_campsite, permissions::{GeneralPermissionConsts, has_role_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[get("/xrpc/gg.campground.membership.getInvites?<campsite_id>&<limit>&<offset>")]
pub async fn get_invites(auth: CampsiteInfo<'_>, campsite_id: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Json<GetCampsiteInvitesOutput>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    if limit < 1 || limit > 100 {
        return Err(XRPCError::BadRequest("Expected limit query to be between and including 1 and 100".to_string()));
    }

    if !has_role_perms_or_owner(&auth.campsite, &auth.member, GeneralPermissionConsts::MANAGE_INVITES, 0).await? {
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
        .inner_join(
            profile::table
                .on(
                    profile::creator.eq(
                        campsite_invite::createdby
                    )
                )
        )
        .inner_join(
            crate::schema::appview::actor::table
                .on(
                    crate::schema::appview::actor::did.eq(
                        campsite_invite::createdby
                    )
                )
        )
        .load::<(CampsiteInvite, Profile, Actor)>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(|x| campsite_invite_view_campsite(&x.0, &x.1, &x.2))
        .collect::<Vec<CampsiteInviteViewCampsite>>();

    return Ok(Json(GetCampsiteInvitesOutput { invites }));
}