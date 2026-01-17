use appview_schema::{models::appview::{Tent, TentMessage}, schema::appview::{tent, tent_message}};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentMessageViewBasic;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::{establish_connection, profiles::get_profile}, helpers::{api::handle_select_first_error, tents::tent_message_view_basic}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.tent.deleteMessage?<tent_id>&<id>")]
pub async fn delete_message(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, tent_id: &str, id: &str) -> Result<Json<TentMessageViewBasic>> {    
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?;

    let mut conn = establish_connection().unwrap();
    let (actor, profile) = get_profile(client, did_document_storage, actor_did.as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    // Can be given invalid UUID; Be descriptive
    let tent_id_uuid = Uuid::try_parse(tent_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'tent_id' query to be a valid UUID".to_string()))
        ?;
    // Check if tent exists first and can be viewed by user, so they can't check if message exists by ID if they are not there
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
    } else if tent_filtered.r#type != 0 {
        return Err(XRPCError::BadRequest("This tent type does not support messages".to_string()));
    }

    let msg_id_uuid = Uuid::try_parse(id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))
        ?;
    let msg = &tent_message::table
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(
                    tent_message::tentid
                        .eq(tent_id_uuid)
                )
            )
            .first::<TentMessage>(&mut conn)
            .map_err(handle_select_first_error)?;

    if msg.created_by != actor_did {
        return Err(XRPCError::Forbidden("Cannot delete message not created by the user".to_string()));
    }

    diesel::delete(tent_message::table)
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(
                    tent_message::tentid
                        .eq(tent_id_uuid)
                )
        )
        .execute(&mut conn)
        .expect("Error deleting message");

    return Ok(Json(tent_message_view_basic(tent_filtered, msg, &Some(actor), &Some(profile))));
}
