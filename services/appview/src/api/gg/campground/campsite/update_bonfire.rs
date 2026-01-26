use appview_schema::{models::appview::Bonfire, schema::appview};
use campground_lexicon::gg::campground::campsite::BonfireViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::bonfire_view_basic, permissions::{CampsitePermissionConsts, TentPermissionConsts, has_tent_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateBonfireBody {
    name: Option<String>,
    description: Option<String>,
    priority: Option<i32>,
}

#[post("/xrpc/gg.campground.campsite.updateBonfire?<campsite_id>&<bonfire_id>", data = "<body>")]
pub async fn update_bonfire(auth: CampsiteInfo<'_>, campsite_id: &str, bonfire_id: &str, body: Json<UpdateBonfireBody>) -> Result<Json<BonfireViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let existing_bonfire = appview::bonfire::table
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(
                    appview::bonfire::id
                        .eq(bonfire_id)
                )
        )
        .first::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    if !has_tent_perms_or_owner(&auth.campsite, &bonfire_id, None, None, &auth.member, CampsitePermissionConsts::MANAGE_BONFIRES, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let current_date = Utc::now().naive_utc();

    let (name, description, priority) = (inner_body.name.clone().unwrap_or(existing_bonfire.name), inner_body.description.clone().unwrap_or(existing_bonfire.description), inner_body.priority.unwrap_or(existing_bonfire.priority));

    let bonfire = diesel::update(appview::bonfire::table)
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(
                    appview::bonfire::id
                        .eq(bonfire_id)
                )
        )
        .set((
            appview::bonfire::name
                .eq(name),
            appview::bonfire::description
                .eq(description),
            appview::bonfire::priority
                .eq(priority),
            appview::bonfire::updatedby
                .eq(&auth.actor.did),
            appview::bonfire::updatedat
                .eq(current_date),
        ))
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    let bonfire_view = bonfire_view_basic(bonfire.first().unwrap());
    return Ok(Json(bonfire_view));
}