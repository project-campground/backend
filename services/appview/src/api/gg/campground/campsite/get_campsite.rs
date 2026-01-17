use appview_schema::models::appview::{Bonfire, Campsite};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::campsite::{BonfireViewBasic, CampsiteViewDetailed};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{database::{actors::get_actor, establish_connection}, helpers::campsites::{bonfire_view_basic, campsite_view_detailed}, xrpc::{
    auth::Authorization,
    error::{Result, XRPCError}
}};

#[get("/xrpc/gg.campground.campsite.getCampsite?<id>")]
pub async fn get_campsite(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, id: &str) -> Result<Json<CampsiteViewDetailed>> {
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?;

    let mut conn = establish_connection().unwrap();
    let actor = &get_actor(client, did_document_storage, actor_did.as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;
    // Not in the campsite to view that
    if !actor.campsites.contains(&Some(id.to_string())) {
        return Err(XRPCError::Forbidden("User cannot view campsite that they are not member of".to_string()));
    }

    let campsite = &crate::schema::appview::campsite::table
        .filter(crate::schema::appview::campsite::id.eq(id))
        .first::<Campsite>(&mut conn)
        .expect("Error loading campsites");
    let bonfires = crate::schema::appview::bonfire::table
        .filter(crate::schema::appview::bonfire::campsiteid.eq(id))
        .load::<Bonfire>(&mut conn)
        .expect("Error loading campsite's bonfire")
        .iter()
        .map(bonfire_view_basic)
        .collect::<Vec<BonfireViewBasic>>();
    let campsite_view = campsite_view_detailed(campsite, bonfires);

    return Ok(Json(campsite_view));
}