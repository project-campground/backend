#![
    allow(unused_variables)
]
use appview_schema::{models::appview::TentMessage, schema::appview::tent_message};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentMessageViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{establish_connection, profiles::get_profile_from_actor}, helpers::{api::handle_select_first_error, permissions::{ContentPermissionConsts, has_leveled_perms_or_owner}, tents::tent_message_view_basic, ws::event_next_tent}, realtime::data::ReactiveSubject, xrpc::{
        campsite::TentInfo, error::{Result, XRPCError}
    }
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateMessageBody {
    content: String,
    replies: Option<Vec<String>>,
}

#[post("/xrpc/gg.campground.tent.createMessage?<tent_id>", data = "<body>")]
pub async fn create_message<'a>(auth: TentInfo<'_>, event_subject: &State<ReactiveSubject>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, tent_id: &str, body: Json<CreateMessageBody>) -> Result<Json<TentMessageViewBasic>> {    
    if body.content.len() > 4000 || body.content.len() == 0 {
        return Err(XRPCError::BadRequest("Expected message content length to be between (and including) 1 and 4000.".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    let (actor, profile) = get_profile_from_actor(client, did_document_storage, auth.actor)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;
    
    if auth.tent.r#type != 0 {
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

    if !has_leveled_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, 0, ContentPermissionConsts::VIEW_CONTENT | ContentPermissionConsts::CREATE_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let replies_query: Vec<TentMessage> = tent_message::table
        .filter(
            crate::schema::appview::tent_message::tentid
                .eq(auth.tent.id)
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
    let messages = &diesel::insert_into(tent_message::table)
        .values(TentMessage {
            id: Uuid::new_v4(),
            campsite_id: auth.tent.campsite_id.clone(),
            r#type: 0,
            components: vec![],
            tent_id: auth.tent.id.clone(),
            content: body.content.clone(),
            replying_to: uuids.iter().map(|x| Some(x.clone())).collect::<Vec<Option<Uuid>>>(),
            created_at: Utc::now().naive_utc(),
            created_by: actor.did.clone(),
            updated_at: None,
        })
        .load::<TentMessage>(&mut conn)
        .map_err(handle_select_first_error)?;

    let first_message = messages.first().unwrap();

    event_next_tent(event_subject, &auth.tent, false, "MessageCreated", tent_message_view_basic(&auth.tent, first_message, &Some(actor.clone()), &Some(profile.clone()), &Some(auth.member.clone())));

    return Ok(Json(tent_message_view_basic(&auth.tent, first_message, &Some(actor), &Some(profile), &Some(auth.member))));
}
