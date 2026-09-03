#![allow(unused_variables)]
use appview_schema::{models::appview::TentMessage, schema::appview::tent_message};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::message::{MessageType, MessageViewBasic};
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{establish_connection, profiles::get_profile_from_actor},
    expect_permission,
    helpers::{
        permissions::{ContentPermissionConsts, has_leveled_perms_or_owner},
        ws::event_next_tent,
    },
    realtime::data::ReactiveSubject,
    views::messages::message_view_basic,
    xrpc::{
        campsite::TentInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateMessageBody {
    content: String,
}

#[post(
    "/xrpc/gg.campground.message.updateMessage?<tent_id>&<message_id>",
    data = "<body>"
)]
pub async fn update_message(
    auth: TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    tent_id: &str,
    message_id: &str,
    body: Json<UpdateMessageBody>,
) -> Result<Json<MessageViewBasic>> {
    if body.content.len() > 4000 || body.content.len() == 0 {
        return Err(XRPCError::BadRequest(
            "Expected message content length to be between (and including) 1 and 4000.".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();
    let (actor, profile) = get_profile_from_actor(client, did_document_storage, auth.actor)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    // Invalid type
    if auth.tent.r#type != 0 {
        return Err(XRPCError::BadRequest(
            "This tent type does not support messages".to_string(),
        ));
    }

    let msg_id_uuid = Uuid::try_parse(message_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))?;
    let msg = &tent_message::table
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(tent_message::tentid.eq(auth.tent.id)),
        )
        .first::<TentMessage>(&mut conn)
        .map_err(|x| match x {
            diesel::result::Error::NotFound => XRPCError::NotFound,
            _ => XRPCError::InternalServerError,
        })?;

    let content = body.content.clone();

    if msg.created_by != actor.did {
        return Err(XRPCError::Forbidden(
            "Cannot update message not created by the user".to_string(),
        ));
    } else if msg.r#type == MessageType::System as i16 {
        return Err(XRPCError::Forbidden(
            "Cannot update system messages".to_string(),
        ));
    } else if content == msg.content {
        return Err(XRPCError::BadRequest(
            "Message already has the same content".to_string(),
        ));
    }

    expect_permission!(has_leveled_perms_or_owner(
        &auth.campsite,
        &auth.tent.bonfire_id,
        auth.tent.category_id.clone(),
        Some(auth.tent.id),
        &auth.member,
        0,
        ContentPermissionConsts::VIEW_CONTENT
    ));

    let updated_at = Utc::now().naive_utc();

    let updated_messages = diesel::update(tent_message::table)
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(tent_message::tentid.eq(auth.tent.id)),
        )
        .set((
            tent_message::content.eq(&body.content),
            tent_message::updatedat.eq(updated_at),
        ))
        .load::<TentMessage>(&mut conn)
        .expect("Error updating message");

    let updated_message = updated_messages.first().unwrap();

    event_next_tent(
        event_subject,
        &auth.tent,
        false,
        "MessageUpdated",
        message_view_basic(
            &auth.tent,
            updated_message,
            Some(&actor),
            profile.as_ref(),
            Some(&auth.member),
        ),
    );

    // New tent message, since it has been updated and is not given by SQL
    return Ok(Json(message_view_basic(
        &auth.tent,
        updated_message,
        Some(&actor),
        profile.as_ref(),
        Some(&auth.member),
    )));
}
