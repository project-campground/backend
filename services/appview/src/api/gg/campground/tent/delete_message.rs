#![
    allow(unused_variables)
]
use appview_schema::{models::appview::TentMessage, schema::appview::tent_message};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentMessageViewBasic;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::{establish_connection, profiles::get_profile_from_actor}, helpers::{api::handle_select_first_error, permissions::{TentPermissionConsts, has_tent_perms_or_owner}, tents::tent_message_view_basic}, xrpc::{
        campsite::TentInfo, error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.tent.deleteMessage?<tent_id>&<message_id>")]
pub async fn delete_message(auth: TentInfo<'_>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, tent_id: &str, message_id: &str) -> Result<Json<TentMessageViewBasic>> {    
    let mut conn = establish_connection().unwrap();
    let (actor, profile) = get_profile_from_actor(client, did_document_storage, auth.actor.clone())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    if auth.tent.r#type != 0 {
        return Err(XRPCError::BadRequest("This tent type does not support messages".to_string()));
    }

    let msg_id_uuid = Uuid::try_parse(message_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))
        ?;
    let msg = &tent_message::table
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(
                    tent_message::tentid
                        .eq(auth.tent.id)
                )
            )
            .first::<TentMessage>(&mut conn)
            .map_err(handle_select_first_error)?;

    let required_perms = if msg.created_by != auth.actor.did { TentPermissionConsts::VIEW_CONTENT | TentPermissionConsts::MANAGE_CONTENT } else { TentPermissionConsts::VIEW_CONTENT };

    if !has_tent_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, 0, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    diesel::delete(tent_message::table)
        .filter(
            tent_message::id
                .eq(msg_id_uuid)
                .and(
                    tent_message::tentid
                        .eq(auth.tent.id)
                )
        )
        .execute(&mut conn)
        .expect("Error deleting message");

    return Ok(Json(tent_message_view_basic(&auth.tent, msg, &Some(actor), &Some(profile))));
}
