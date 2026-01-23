use appview_schema::{models::appview::TentCategory, schema::appview::{campsite_permission, tent, tent_category}};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentCategoryView;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::{actors::get_actor, establish_connection}, helpers::{api::handle_select_first_error, tents::tent_category_view}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.tent.deleteCategory?<category_id>")]
pub async fn delete_category(auth: Authorization<'_>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, category_id: &str) -> Result<Json<TentCategoryView>> {    
    let actor_did = auth.actor_did;

    let mut conn = establish_connection().unwrap();
    let actor = get_actor(client, did_document_storage, actor_did.as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    // Can be given invalid UUID; Be descriptive
    let category_id_uuid = Uuid::try_parse(category_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'category_id' query to be a valid UUID".to_string()))
        ?;
    let category = &tent_category::table
        .filter(
            tent_category::id
                .eq(category_id_uuid)
        )
        .first::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;

    // Not in the campsite to view that
    if !actor.campsites.contains(&Some(category.campsite_id.to_string())) {
        return Err(XRPCError::Forbidden("User cannot view campsite that they are not member of".to_string()));
    }

    diesel::delete(tent_category::table)
        .filter(
            tent_category::id
                .eq(category.id)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;
    diesel::delete(campsite_permission::table)
        .filter(
            campsite_permission::tentid
                .eq(category_id_uuid)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;

    // To make it easier to delete sections of tents
    diesel::delete(tent::table)
        .filter(
            tent::categoryid
                .eq(category.id)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;

    return Ok(Json(tent_category_view(category)));
}
