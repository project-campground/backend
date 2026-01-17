use appview_schema::models::appview::Bonfire;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::campsite::BonfireViewBasic;
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;
use serde::Deserialize;

use crate::{database::{actors::get_actor, establish_connection}, helpers::campsites::bonfire_view_basic, xrpc::{
    auth::Authorization,
    error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateBonfireBody {
    name: String,
    description: String,
    priority: i32,
}

#[post("/xrpc/gg.campground.campsite.createBonfire?<campsite_id>", data = "<body>")]
pub async fn create_bonfire(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, campsite_id: &str, body: Json<CreateBonfireBody>) -> Result<Json<BonfireViewBasic>> {    
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?.clone();

    let inner_body = &body.into_inner();
    if inner_body.name.len() < 3 || inner_body.name.len() > 48 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.len() > 200 {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    let actor = &get_actor(client, did_document_storage, actor_did.clone().as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let campsite_count = crate::schema::appview::campsite::table
        .filter(crate::schema::appview::campsite::id.eq(campsite_id))
        .execute(&mut conn)
        .expect("Error loading campsites");

    if campsite_count < 1 {
        return Err(XRPCError::NotFound);
    }

    let existing_bonfire_count = crate::schema::appview::bonfire::table
        .filter(crate::schema::appview::bonfire::campsiteid.eq(campsite_id))
        .execute(&mut conn)
        .expect("Error loading bonfires");
    
    if existing_bonfire_count >= 20 {
        return Err(XRPCError::Forbidden("Cannot create more than 20 bonfires in a campsite".to_string()));
    }

    let current_date = Utc::now().naive_utc();

    let bonfire = &diesel::insert_into(crate::schema::appview::bonfire::table)
        .values(
            Bonfire {
                id: campsite_id.to_string(),
                campsite_id: campsite_id.to_string(),
                name: inner_body.name.clone(),
                description: inner_body.description.clone(),
                avatar_uri: None,
                banner_uri: None,
                priority: inner_body.priority,
                created_by: actor.did.clone(),
                created_at: current_date,
                updated_by: actor.did.clone(),
                updated_at: current_date,
            }
        )
        .get_result::<Bonfire>(&mut conn)
        .expect("Error inserting bonfire");

    let bonfire_view = bonfire_view_basic(bonfire);

    return Ok(Json(bonfire_view));
}