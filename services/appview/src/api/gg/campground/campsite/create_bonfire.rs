use appview_schema::models::appview::Bonfire;
use campground_lexicon::gg::campground::campsite::BonfireViewBasic;
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{State, serde::json::Json};
use rsky_common::tid::Ticker;
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{api::handle_all_db_errors, campsites::bonfire_view_basic, permissions::{GeneralPermissionConsts, has_role_perms_or_owner}, ws::event_next_bonfire}, realtime::data::ReactiveSubject, util::params::ensure_valid_set_uri, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateBonfireBody<'a> {
    name: String,
    description: String,
    priority: i32,
    avatar_uri: Option<&'a str>,
    banner_uri: Option<&'a str>,
}

#[post("/xrpc/gg.campground.campsite.createBonfire?<campsite_id>", data = "<body>")]
pub async fn create_bonfire(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, body: Json<CreateBonfireBody<'_>>) -> Result<Json<BonfireViewBasic>> {    
    let CreateBonfireBody { name, description, priority, avatar_uri, banner_uri } = &body.into_inner();
    if name.len() < 3 || name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if description.len() > 200 {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    } else if !has_role_perms_or_owner(&auth.campsite, &auth.member, GeneralPermissionConsts::MANAGE_BONFIRES, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }
    let avatar_uri = &ensure_valid_set_uri(avatar_uri)
        .map_err(|x|
            XRPCError::BadRequest(x.to_string())
        )?
        .map(|x| x.to_string());
    let banner_uri = &ensure_valid_set_uri(banner_uri)
        .map_err(|x|
            XRPCError::BadRequest(x.to_string())
        )?
        .map(|x| x.to_string());

    let mut conn = establish_connection().unwrap();

    let existing_bonfire_count = crate::schema::appview::bonfire::table
        .filter(crate::schema::appview::bonfire::campsiteid.eq(campsite_id))
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_all_db_errors)?;
    
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
                name: name.clone(),
                description: description.clone(),
                avatar_uri: avatar_uri.clone(),
                banner_uri: banner_uri.clone(),
                priority: *priority,
                created_by: auth.actor.did.clone(),
                created_at: current_date,
                updated_by: auth.actor.did,
                updated_at: current_date,
            }
        )
        .get_result::<Bonfire>(&mut conn)
        .expect("Error inserting bonfire");

    event_next_bonfire(event_subject, &bonfire, false, "BonfireCreated", bonfire_view_basic(bonfire));

    return Ok(Json(bonfire_view_basic(bonfire)));
}