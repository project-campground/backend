use appview_schema::{models::appview::{Tent, TentMessage}, schema::appview::tent_message};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentMessageViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{establish_connection, profiles::get_profile}, helpers::{api::handle_select_first_error, tents::tent_message_view_basic}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateMessageBody {
    content: String,
    replies: Option<Vec<String>>,
}

#[post("/xrpc/gg.campground.tent.createMessage?<tent_id>", data = "<body>")]
pub async fn create_message(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, tent_id: &str, body: Json<CreateMessageBody>) -> Result<Json<TentMessageViewBasic>> {    
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
    let tent = &crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::id
                .eq(tent_id_uuid)
        )
        .first::<Tent>(&mut conn)
        .map_err(handle_select_first_error)?;

    // Not in the campsite to view that
    if !actor.campsites.contains(&Some(tent.campsite_id.to_string())) {
        return Err(XRPCError::Forbidden("User cannot view campsite that they are not member of".to_string()));
    } else if tent.r#type != 0 {
        return Err(XRPCError::BadRequest("This tent type does not support messages".to_string()));
    }

    let replies = body.replies.clone().unwrap_or(vec![]);

    if replies.len() > 5 {
        return Err(XRPCError::BadRequest("Expected no more than 5 elements in 'replies' array".to_string()));
    }

    let uuids: Vec<Uuid> = replies.iter().filter_map(|y| Uuid::try_parse(y.as_str()).ok()).collect();

    if uuids.len() < replies.len() {
        return Err(XRPCError::BadRequest("Expected 'replies' parameters to have valid UUIDs".to_string()));
    }

    let replies_query: Vec<TentMessage> = tent_message::table
        .filter(
            crate::schema::appview::tent_message::tentid
                .eq(tent_id_uuid)
                .and(
                    crate::schema::appview::tent_message::id
                        .eq_any(uuids.clone())
                )
        )
        .load::<TentMessage>(&mut conn)
        .expect("Error loading tent messages");

    if replies_query.len() < uuids.clone().len() {
        return Err(XRPCError::BadRequest("Some of the messages being replied to no longer exist or never existed in this tent.".to_string()));
    }

    let message = &diesel::insert_into(tent_message::table)
        .values(TentMessage {
            id: Uuid::new_v4(),
            campsite_id: tent.campsite_id.clone(),
            tent_id: tent.id,
            content: body.content.clone(),
            replying_to: uuids.iter().map(|x| Some(x.clone())).collect::<Vec<Option<Uuid>>>(),
            created_at: Utc::now().naive_utc(),
            created_by: actor_did,
            updated_at: None,
        })
        .load::<TentMessage>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(tent_message_view_basic(tent, message.first().unwrap(), &Some(actor), &Some(profile))));
}
