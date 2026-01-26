use appview_schema::{models::appview::{Campsite, CampsiteInvite}, schema::appview::{self, campsite_invite}};
use campground_lexicon::gg::campground::campsite::CampsiteInviteViewDetailed;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_invite_view_detailed}, xrpc::error::{Result, XRPCError}};

#[get("/xrpc/gg.campground.membership.getInvite?<invite_id>")]
pub async fn get_invite(invite_id: &str) -> Result<Json<CampsiteInviteViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let uuid = Uuid::try_parse(invite_id)
        .map_err(|_| XRPCError::BadRequest("Invalid 'invite_id' format. Expected UUID".to_string()))?;

    let invite = campsite_invite::table
        .filter(campsite_invite::id.eq(uuid))
        .first::<CampsiteInvite>(&mut conn)
        .map_err(handle_select_first_error)?;

    let campsite = appview::campsite::table
        .filter(appview::campsite::id.eq(invite.campsite_id.clone()))
        .first::<Campsite>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(campsite_invite_view_detailed(&invite, &campsite)));
}