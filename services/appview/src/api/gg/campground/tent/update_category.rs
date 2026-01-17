use appview_schema::{models::appview::{Bonfire, Tent, TentCategory}, schema::appview};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentCategoryView;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{actors::get_actor, establish_connection}, helpers::{api::handle_select_first_error, tents::tent_category_view}, xrpc::{
    auth::Authorization,
    error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateCategoryBody {
    name: Option<String>,
    description: Option<String>,
    bonfire_id: Option<String>,
    priority: Option<i32>,
}

#[post("/xrpc/gg.campground.tent.updateCategory?<id>", data = "<body>")]
pub async fn update_category(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, id: &str, body: Json<UpdateCategoryBody>) -> Result<Json<TentCategoryView>> {    
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?.clone();

    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    // Can be given invalid UUID; Be descriptive
    let category_id_uuid = Uuid::try_parse(id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))
        ?;

    let mut conn = establish_connection().unwrap();
    let actor = &get_actor(client, did_document_storage, actor_did.clone().as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let category = &crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::id
                .eq(category_id_uuid)
        )
        .first::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;

    let moved_bonfire = inner_body.bonfire_id.clone().unwrap_or(category.bonfire_id.clone());

    // To make sure they are not moving to category that doesn't exist
    if moved_bonfire != category.bonfire_id {
        check_bonfire_existence(category.campsite_id.clone(), moved_bonfire.clone())
            .await?;
    }
    let current_date = Utc::now().naive_utc();
    
    let updated_category = diesel::update(crate::schema::appview::tent_category::table)
        .filter(
            appview::tent_category::id
                .eq(
                    category.id
                )
        )
        .set((
            // All the new settings
            appview::tent_category::name
                .eq(inner_body.name.clone().unwrap_or(category.name.clone())),
            appview::tent_category::description
                .eq(inner_body.description.clone().unwrap_or(category.description.clone())),
            appview::tent_category::priority
                .eq(inner_body.priority.clone().unwrap_or(category.priority.clone())),
            appview::tent_category::bonfireid
                .eq(moved_bonfire.clone()),
            // Mandatory
            appview::tent_category::updatedat
                .eq(current_date),
            appview::tent_category::updatedby
                .eq(actor.did.clone()),
        ))
        .load::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    // If category was moved to another bonfire, then move all the tents along with it
    if moved_bonfire != category.bonfire_id {
        diesel::update(crate::schema::appview::tent::table)
            .filter(
                appview::tent::categoryid
                    .eq(
                        category.id
                    )
            )
            .set((
                // All the new settings
                appview::tent::bonfireid
                    .eq(moved_bonfire),
            ))
            .load::<Tent>(&mut conn)
            .map_err(handle_select_first_error)?;
    }

    return Ok(Json(tent_category_view(updated_category.first().unwrap())));
}

async fn check_bonfire_existence(campsite_id: String, moved_bonfire_id: String) -> Result<(), XRPCError> {    
    let mut conn = establish_connection().unwrap();

    let bonfires = &crate::schema::appview::bonfire::table
        .filter(
            crate::schema::appview::bonfire::id
                .eq(moved_bonfire_id)
                .and(
                    crate::schema::appview::bonfire::campsiteid
                        .eq(
                            campsite_id
                        )
                )
        )
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    if bonfires.len() == 0 {
        return Err(XRPCError::BadRequest("Bonfire supplied in 'bonfire_id' does not exist".to_string()));
    }

    return Ok(());
}