use appview_schema::{models::appview::TentCategory, schema::appview};
use campground_lexicon::gg::campground::tent::TentCategoryView;
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, tents::tent_category_view}, xrpc::{
    campsite::CategoryInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateCategoryBody {
    name: Option<String>,
    description: Option<String>,
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.updateCategory?<category_id>", data = "<body>")]
pub async fn update_category(auth: CategoryInfo<'_>, category_id: &str, body: Json<UpdateCategoryBody>) -> Result<Json<TentCategoryView>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let current_date = Utc::now().naive_utc();
    
    let updated_category = diesel::update(crate::schema::appview::tent_category::table)
        .filter(
            appview::tent_category::id
                .eq(
                    auth.category.id
                )
        )
        .set((
            // All the new settings
            appview::tent_category::name
                .eq(inner_body.name.clone().unwrap_or(auth.category.name.clone())),
            appview::tent_category::description
                .eq(inner_body.description.clone().unwrap_or(auth.category.description.clone())),
            // Mandatory
            appview::tent_category::updatedat
                .eq(current_date),
            appview::tent_category::updatedby
                .eq(&auth.actor.did),
        ))
        .load::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(tent_category_view(updated_category.first().unwrap())));
}
