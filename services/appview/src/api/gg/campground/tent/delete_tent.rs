use appview_schema::{models::appview::Tent, schema::appview::tent};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentViewBasic;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::{actors::get_actor, establish_connection}, helpers::{api::handle_select_first_error, tents::tent_view_basic}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.tent.deleteTent?<id>")]
pub async fn delete_tent(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, id: &str) -> Result<Json<TentViewBasic>> {    
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?;

    let mut conn = establish_connection().unwrap();
    let actor = get_actor(client, did_document_storage, actor_did.as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    // Can be given invalid UUID; Be descriptive
    let tent_id_uuid = Uuid::try_parse(id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))
        ?;

    let tent_filtered = &tent::table
        .filter(
            tent::id
                .eq(tent_id_uuid)
        )
        .first::<Tent>(&mut conn)
        .map_err(handle_select_first_error)?;

    // Not in the campsite to view that
    if !actor.campsites.contains(&Some(tent_filtered.campsite_id.to_string())) {
        return Err(XRPCError::Forbidden("User cannot view campsite that they are not member of".to_string()));
    }

    diesel::delete(tent::table)
        .filter(
            tent::id
                .eq(tent_filtered.id)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;

    return Ok(Json(tent_view_basic(tent_filtered)));
}
