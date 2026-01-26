use appview_schema::models::appview::Bonfire;
use campground_lexicon::gg::campground::campsite::BonfireViewBasic;
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use rsky_common::tid::Ticker;
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{campsites::bonfire_view_basic, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateBonfireBody {
    name: String,
    description: String,
    priority: i32,
}

#[post("/xrpc/gg.campground.campsite.createBonfire?<campsite_id>", data = "<body>")]
pub async fn create_bonfire(auth: CampsiteInfo<'_>, campsite_id: &str, body: Json<CreateBonfireBody>) -> Result<Json<BonfireViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.len() < 3 || inner_body.name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    }
    else if inner_body.description.len() > 200 {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    if !has_role_perms_or_owner(auth.campsite, auth.member.clone(), CampsitePermissionConsts::MANAGE_BONFIRES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let existing_bonfire_count = crate::schema::appview::bonfire::table
        .filter(crate::schema::appview::bonfire::campsiteid.eq(campsite_id))
        .execute(&mut conn)
        .expect("Error loading bonfires");
    
    if existing_bonfire_count >= 20 {
        return Err(XRPCError::Forbidden("Cannot create more than 20 bonfires in a campsite".to_string()));
    }

    let current_date = Utc::now().naive_utc();

    let mut ticker = Ticker::new();
    let bonfire_id = ticker.next(None);

    let bonfire = &diesel::insert_into(crate::schema::appview::bonfire::table)
        .values(
            Bonfire {
                id: bonfire_id.to_string(),
                campsite_id: campsite_id.to_string(),
                name: inner_body.name.clone(),
                description: inner_body.description.clone(),
                avatar_uri: None,
                banner_uri: None,
                priority: inner_body.priority,
                created_by: auth.actor.did.clone(),
                created_at: current_date,
                updated_by: auth.actor.did.clone(),
                updated_at: current_date,
            }
        )
        .get_result::<Bonfire>(&mut conn)
        .expect("Error inserting bonfire");

    let bonfire_view = bonfire_view_basic(bonfire);

    return Ok(Json(bonfire_view));
}