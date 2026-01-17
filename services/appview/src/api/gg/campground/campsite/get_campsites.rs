use appview_schema::models::appview::Campsite;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::campsite::{CampsiteViewBasic, GetCampsitesOutput};
use diesel::{ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{
    database::establish_connection, helpers::{campsites::campsite_view_basic, deduplicate_list, lower_list}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.campsite.getCampsites?<ids>")]
pub async fn get_campsites(_auth: Authorization, _client: &State<Client>, _did_document_storage: &State<LruDidDocumentStorage>, ids: Vec<&str>) -> Result<Json<GetCampsitesOutput>> {
    let mut conn = establish_connection().unwrap();

    let ids = deduplicate_list(lower_list(ids));

    if ids.len() > 25 || ids.len() == 0 {
        return Err(XRPCError::BadRequest("ids query must have at least 1 campsite and less than or equal to 25".to_string()));
    }

    let campsites = crate::schema::appview::campsite::table
        .filter(crate::schema::appview::campsite::id.eq_any(ids))
        .load::<Campsite>(&mut conn)
        .expect("Error loading campsites")
        .iter()
        .map(campsite_view_basic)
        .collect::<Vec<CampsiteViewBasic>>();

    return Ok(Json(GetCampsitesOutput { campsites: campsites }));
}