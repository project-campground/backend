#![
    allow(unused_variables)
]
use appview_schema::{models::appview::{Bonfire, Tent, TentCategory}, schema::appview};
use campground_lexicon::gg::campground::tent::TentViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, roles::{CampsitePermissionConsts, has_tent_perms_or_owner}, tents::tent_view_basic}, xrpc::{
    campsite::TentInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateTentBody {
    name: Option<String>,
    description: Option<String>,
    bonfire_id: Option<String>,
    category_id: Option<String>,
    priority: Option<i32>,
}

#[post("/xrpc/gg.campground.tent.updateTent?<tent_id>", data = "<body>")]
pub async fn update_tent(auth: TentInfo<'_>, tent_id: &str, body: Json<UpdateTentBody>) -> Result<Json<TentViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    if !has_tent_perms_or_owner(auth.campsite.clone(), auth.tent.bonfire_id.clone(), auth.tent.category_id.clone(), Some(auth.tent.id), auth.member.clone(), CampsitePermissionConsts::MANAGE_TENTS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    let remove_category = inner_body.category_id.clone().map_or(false, |x| x == "");
    let category_id =
        if inner_body.category_id.is_none() || remove_category {
            None
        } else {
            Some(Uuid::try_parse(inner_body.category_id.clone().unwrap().as_str())
                .map_err(|_| XRPCError::BadRequest("Invalid category_id UUID format".to_string()))?)
        };

    let mut conn = establish_connection().unwrap();

    let moved_bonfire = inner_body.bonfire_id.clone().unwrap_or(auth.tent.bonfire_id.clone());

    // To make sure they are not moving to category that doesn't exist
    if !remove_category && category_id.map_or(false, |x| Some(x) != auth.tent.category_id) {
        check_category_existence(auth.tent.campsite_id.clone(), moved_bonfire.clone(), category_id.unwrap()).await?;
    } else if moved_bonfire != auth.tent.bonfire_id {
        check_bonfire_existence(auth.tent.campsite_id.clone(), moved_bonfire.clone()).await?
    }
    let moved_category = if remove_category { None } else { category_id.or(auth.tent.category_id.clone()) };

    let current_date = Utc::now().naive_utc();
    
    let updated_tent = diesel::update(crate::schema::appview::tent::table)
        .filter(
            appview::tent::id
                .eq(
                    auth.tent.id.clone()
                )
        )
        .set((
            // All the new settings
            appview::tent::name
                .eq(inner_body.name.clone().unwrap_or(auth.tent.name)),
            appview::tent::description
                .eq(inner_body.description.clone().unwrap_or(auth.tent.description)),
            appview::tent::priority
                .eq(inner_body.priority.clone().unwrap_or(auth.tent.priority)),
            appview::tent::categoryid
                .eq(moved_category),
            appview::tent::bonfireid
                .eq(moved_bonfire.clone()),
            // Mandatory
            appview::tent::updatedat
                .eq(current_date),
            appview::tent::updatedby
                .eq(auth.actor.did.clone()),
        ))
        .load::<Tent>(&mut conn)
        .expect("Error updating tent");

    return Ok(Json(tent_view_basic(updated_tent.first().unwrap())));
}

async fn check_category_existence(campsite_id: String, moved_bonfire_id: String, category_id: Uuid) -> Result<String, XRPCError> {    
    let mut conn = establish_connection().unwrap();

    let tent_categories = &crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::id
                .eq(category_id)
        )
        .load::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;

    if tent_categories.len() == 0 {
        return Err(XRPCError::BadRequest("Category supplied in 'category_id' does not exist".to_string()));
    }

    let tent_category = tent_categories.first().unwrap();

    if tent_category.campsite_id != campsite_id {
        return Err(XRPCError::BadRequest("Category supplied in 'category_id' does not belong to the same campsite as tent".to_string()));
    } else if tent_category.bonfire_id != moved_bonfire_id {
        return Err(XRPCError::BadRequest("Category supplied in 'category_id' does not belong to the same bonfire as supplied in 'bonfire_id' or already existing bonfire".to_string()));
    }

    return Ok(tent_category.bonfire_id.clone());
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