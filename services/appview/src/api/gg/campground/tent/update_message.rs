use appview_schema::{models::appview::{Tent, TentMessage}, schema::appview::{tent, tent_message}};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentMessageViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{establish_connection, profiles::get_profile}, helpers::tents::tent_message_view_basic, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateMessageBody {
    content: String,
}

#[post("/xrpc/gg.campground.tent.updateMessage?<tent_id>&<id>", data = "<body>")]
pub async fn update_message(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, tent_id: &str, id: &str, body: Json<UpdateMessageBody>) -> Result<Json<TentMessageViewBasic>> {    
    if body.content.len() > 4000 || body.content.len() == 0 {
        return Err(XRPCError::BadRequest("Expected message content length to be between (and including) 1 and 4000.".to_string()));
    }

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
        .expect("Error loading tent");

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
        .map_err(|x|
            match x {
                diesel::result::Error::NotFound => XRPCError::NotFound,
                _ => XRPCError::InternalServerError,
            }
        )?;

    let content = body.content.clone();

    if msg.created_by != actor_did {
        return Err(XRPCError::Forbidden("Cannot update message not created by the user".to_string()));
    } else if content == msg.content {
        return Err(XRPCError::BadRequest("Message already has the same content".to_string()));
    }

    let updated_at = Utc::now().naive_utc();

    let updated_messages = diesel::update(tent_message::table)
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(
                    tent_message::tentid
                        .eq(tent_id_uuid)
                )
        )
        .set((
            tent_message::content
                .eq(body.content.clone()),
            tent_message::updatedat
                .eq(updated_at)
        ))
        .load::<TentMessage>(&mut conn)
        .expect("Error updating message");

    // New tent message, since it has been updated and is not given by SQL
    return Ok(Json(tent_message_view_basic(
        tent_filtered,
        updated_messages.first().unwrap(),
        &Some(actor),
        &Some(profile)
    )));
}
