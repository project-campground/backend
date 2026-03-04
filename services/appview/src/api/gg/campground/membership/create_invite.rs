use appview_schema::{models::appview::CampsiteInvite, schema::appview::campsite_invite};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::membership::CampsiteInviteViewCampsite;
use chrono::{NaiveDateTime, Utc};
use diesel::RunQueryDsl;
use reqwest::Client;
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{establish_connection, profiles::get_profile_from_actor}, helpers::{api::handle_select_first_error, campsites::campsite_invite_view_campsite, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}, ws::event_next}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateInviteBody {
    expires_at: Option<NaiveDateTime>,
    allowed_amount: Option<i32>,
}

#[post("/xrpc/gg.campground.membership.createInvite?<campsite_id>", data = "<body>")]
pub async fn create_invite(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, campsite_id: &str, body: Json<CreateInviteBody>) -> Result<Json<CampsiteInviteViewCampsite>> {    
    let inner_body = &body.into_inner();
    let current_date = Utc::now().naive_utc();
    if inner_body.allowed_amount.map_or(false, |x| x < 1 || x > 1000) {
        return Err(XRPCError::BadRequest("Expected 'allowedAmount' property to be an integer between 1 and 1000. Create invite without 'allowedAmount' for infinite invite".to_string()));
    } else if inner_body.expires_at.map_or(false, |x| x < current_date) {
        return Err(XRPCError::BadRequest("Expected 'expires_at' property to not result in already expired invite".to_string()));
    }

    if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::CREATE_INVITES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    let (actor, profile) = get_profile_from_actor(client, did_document_storage, auth.actor.clone())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let invite = &diesel::insert_into(campsite_invite::table)
        .values(
            CampsiteInvite {
                id: Uuid::new_v4(),
                campsite_id: campsite_id.to_string(),
                allowed_amount: inner_body.allowed_amount,
                expires_at: inner_body.expires_at,
                created_by: auth.actor.did,
                created_at: current_date,
                used: 0,
            }
        )
        .get_result::<CampsiteInvite>(&mut conn)
        .map_err(handle_select_first_error)?;

    event_next(event_subject, &auth.campsite.id, "InviteCreated", campsite_invite_view_campsite(invite, &profile, &actor));

    return Ok(Json(campsite_invite_view_campsite(invite, &profile, &actor)));
}