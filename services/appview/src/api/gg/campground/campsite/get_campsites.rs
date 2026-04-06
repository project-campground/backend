use appview_schema::models::appview::Campsite;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::campsite::{CampsiteViewBasic, GetCampsitesOutput};
use diesel::{ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::{
    database::establish_connection,
    helpers::{deduplicate_list, lower_list},
    views::campsites::campsite_view_basic,
    xrpc::{
        auth::Authorization,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/gg.campground.campsite.getCampsites?<campsite_ids>")]
pub async fn get_campsites(
    _auth: Authorization<'_>,
    _client: &State<Client>,
    _did_document_storage: &State<LruDidDocumentStorage>,
    campsite_ids: Vec<&str>,
) -> Result<Json<GetCampsitesOutput>> {
    let mut conn = establish_connection().unwrap();

    let campsite_ids = deduplicate_list(lower_list(campsite_ids));

    if campsite_ids.len() > 25 || campsite_ids.len() == 0 {
        return Err(XRPCError::BadRequest(
            "ids query must have at least 1 campsite and less than or equal to 25".to_string(),
        ));
    }

    let campsites = crate::schema::appview::campsite::table
        .filter(crate::schema::appview::campsite::id.eq_any(campsite_ids))
        .load::<Campsite>(&mut conn)
        .expect("Error loading campsites")
        .iter()
        .map(campsite_view_basic)
        .collect::<Vec<CampsiteViewBasic>>();

    return Ok(Json(GetCampsitesOutput {
        campsites: campsites,
    }));
}
