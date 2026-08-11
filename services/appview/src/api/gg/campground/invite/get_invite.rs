use appview_schema::{
    models::appview::{Actor, Campsite, CampsiteInvite, Profile},
    schema::appview::{self, campsite_invite, profile},
};
use campground_lexicon::gg::campground::invite::CampsiteInviteViewGlobal;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection,
    helpers::api::handle_select_first_error,
    views::invites::campsite_invite_view_global,
    xrpc::error::{Result, XRPCError},
};

#[get("/xrpc/gg.campground.invite.getInvite?<invite_id>")]
pub async fn get_invite(invite_id: &str) -> Result<Json<CampsiteInviteViewGlobal>> {
    let mut conn = establish_connection().unwrap();

    let uuid = Uuid::try_parse(invite_id).map_err(|_| {
        XRPCError::BadRequest("Invalid 'invite_id' format. Expected UUID".to_string())
    })?;

    let (invite, actor, profile) = campsite_invite::table
        .filter(campsite_invite::id.eq(uuid))
        .inner_join(
            crate::schema::appview::actor::table
                .on(crate::schema::appview::actor::did.eq(campsite_invite::createdby)),
        )
        .left_join(profile::table.on(profile::creator.eq(campsite_invite::createdby)))
        .first::<(CampsiteInvite, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    let campsite = appview::campsite::table
        .filter(appview::campsite::id.eq(&invite.campsite_id))
        .first::<Campsite>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(campsite_invite_view_global(
        &invite,
        &campsite,
        profile.as_ref(),
        &actor,
    )));
}
