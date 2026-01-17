use appview_schema::{models::appview::Bonfire, schema::appview};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::campsite::BonfireViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;
use serde::Deserialize;

use crate::{database::{actors::get_actor, establish_connection}, helpers::{api::handle_select_first_error, campsites::bonfire_view_basic}, xrpc::{
    auth::Authorization,
    error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateBonfireBody {
    name: Option<String>,
    description: Option<String>,
    priority: Option<i32>,
}

#[post("/xrpc/gg.campground.campsite.updateBonfire?<campsite_id>&<id>", data = "<body>")]
pub async fn update_bonfire(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, campsite_id: &str, id: &str, body: Json<UpdateBonfireBody>) -> Result<Json<BonfireViewBasic>> {    
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?.clone();

    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
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

    let existing_bonfire = appview::bonfire::table
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(
                    appview::bonfire::id
                        .eq(id)
                )
        )
        .first::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;
        
    let current_date = Utc::now().naive_utc();

    let (name, description, priority) = (inner_body.name.clone().unwrap_or(existing_bonfire.name), inner_body.description.clone().unwrap_or(existing_bonfire.description), inner_body.priority.unwrap_or(existing_bonfire.priority));
    
    let bonfire = diesel::update(appview::bonfire::table)
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(
                    appview::bonfire::id
                        .eq(id)
                )
        )
        .set((
            appview::bonfire::name
                .eq(name.clone()),
            appview::bonfire::description
                .eq(description.clone()),
            appview::bonfire::priority
                .eq(priority.clone()),
            appview::bonfire::updatedby
                .eq(actor.did.clone()),
            appview::bonfire::updatedat
                .eq(current_date),
        ))
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    let bonfire_view = bonfire_view_basic(bonfire.first().unwrap());
    return Ok(Json(bonfire_view));
}