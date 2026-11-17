use campground_lexicon::chat::campground::invite::CampsiteInviteViewGlobal;
use cds_schema::{
    models::cds::{Actor, Campsite, CampsiteInvite, Profile},
    schema::cds::{self, campsite_invite, profile},
};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection,
    helpers::api::handle_select_first_error,
    views::invites::campsite_invite_view_global,
    xrpc::error::{Result, XRPCError},
};

#[get("/xrpc/chat.campground.invite.getInvite?<invite_id>")]
pub async fn get_invite(invite_id: &str) -> Result<Json<CampsiteInviteViewGlobal>> {
    let mut conn = establish_connection().unwrap();

    let uuid = Uuid::try_parse(invite_id).map_err(|_| {
        XRPCError::BadRequest("Invalid 'invite_id' format. Expected UUID".to_string())
    })?;

    let (invite, actor, profile) = campsite_invite::table
        .filter(campsite_invite::id.eq(uuid))
        .inner_join(
            crate::schema::cds::actor::table
                .on(crate::schema::cds::actor::did.eq(campsite_invite::created_by)),
        )
        .left_join(profile::table.on(profile::actor.eq(campsite_invite::created_by)))
        .first::<(CampsiteInvite, Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_select_first_error)?;

    let campsite = cds::campsite::table
        .filter(cds::campsite::id.eq(&invite.campsite))
        .first::<Campsite>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(campsite_invite_view_global(
        &invite,
        &campsite,
        profile.as_ref(),
        &actor,
    )));
}
