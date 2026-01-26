use appview_schema::models::appview::TentCategory;
use campground_lexicon::gg::campground::tent::TentCategoryView;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, tents::tent_category_view}, xrpc::{
    campsite::CampsiteInfoBasic, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateCategoryBody {
    name: String,
    description: String,
    priority: i32,
}

#[post("/xrpc/gg.campground.tent.createCategory?<campsite_id>&<bonfire_id>", data = "<body>")]
pub async fn create_category(auth: CampsiteInfoBasic<'_>, campsite_id: &str, bonfire_id: &str, body: Json<CreateCategoryBody>) -> Result<Json<TentCategoryView>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.len() < 3 || inner_body.name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.len() > 200 {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let bonfire_count = crate::schema::appview::bonfire::table
        .filter(
            crate::schema::appview::bonfire::id
                .eq(bonfire_id)
                .and(
                    crate::schema::appview::bonfire::campsiteid
                        .eq(campsite_id)
                )
        )
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;

    if bonfire_count < 1 {
        return Err(XRPCError::NotFound);
    }

    let existing_category_count = crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::tent_category::bonfireid
                        .eq(bonfire_id)
                )
        )
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    if existing_category_count >= 25 {
        return Err(XRPCError::Forbidden("Cannot create more than 25 tent categories in a bonfire".to_string()));
    }

    let current_date = Utc::now().naive_utc();

    let category = &diesel::insert_into(crate::schema::appview::tent_category::table)
        .values(
            TentCategory {
                id: Uuid::new_v4(),
                campsite_id: campsite_id.to_string(),
                bonfire_id: bonfire_id.to_string(),
                name: inner_body.name.clone(),
                description: inner_body.description.clone(),
                priority: inner_body.priority,
                created_by: auth.actor.did.clone(),
                created_at: current_date,
                updated_by: auth.actor.did,
                updated_at: current_date,
            }
        )
        .get_result::<TentCategory>(&mut conn)
        .expect("Error inserting bonfire");

    let category_view = tent_category_view(category);

    return Ok(Json(category_view));
}