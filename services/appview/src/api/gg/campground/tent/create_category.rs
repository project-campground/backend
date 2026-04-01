use appview_schema::models::appview::TentCategory;
use campground_lexicon::gg::campground::tent::TentCategoryView;
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::establish_connection, expect_permission, helpers::{api::handle_select_first_error, permissions::{ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner}, tents::tent_category_view, ws::event_next_category}, realtime::data::ReactiveSubject, xrpc::{
    campsite::BonfireInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateCategoryBody {
    name: String,
    description: String,
    position: i32,
}

#[post("/xrpc/gg.campground.tent.createCategory?<bonfire_id>", data = "<body>")]
pub async fn create_category(auth: BonfireInfo<'_>, event_subject: &State<ReactiveSubject>, bonfire_id: &str, body: Json<CreateCategoryBody>) -> Result<Json<TentCategoryView>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.len() < 3 || inner_body.name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.len() > 200 {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }
    
    let mut conn = establish_connection().unwrap();
    
    let existing_category_count = crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::campsiteid
                .eq(&auth.campsite.id)
        )
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    if existing_category_count >= 25 {
        return Err(XRPCError::Forbidden("Cannot create more than 25 tent categories in a bonfire".to_string()));
    }

    expect_permission!(
        has_leveled_perms_or_owner(&auth.campsite, &auth.bonfire.id, None, None, &auth.member, GeneralPermissionConsts::MANAGE_TENTS, ContentPermissionConsts::VIEW_CONTENT)
    );
    
    let current_date = Utc::now().naive_utc();

    let category = &diesel::insert_into(crate::schema::appview::tent_category::table)
        .values(
            TentCategory {
                id: Uuid::new_v4(),
                campsite_id: auth.campsite.id.clone(),
                bonfire_id: bonfire_id.to_string(),
                name: inner_body.name.clone(),
                description: inner_body.description.clone(),
                priority: inner_body.position,
                created_by: auth.actor.did.clone(),
                created_at: current_date,
                updated_by: auth.actor.did,
                updated_at: current_date,
            }
        )
        .get_result::<TentCategory>(&mut conn)
        .expect("Error inserting bonfire");

    event_next_category(event_subject, &category, false, "CategoryCreated", tent_category_view(&category));

    return Ok(Json(tent_category_view(category)));
}