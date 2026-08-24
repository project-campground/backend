use appview_schema::models::appview::Campsite;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::campsite::{CampsiteViewBasic, GetCampsitesOutput};
use diesel::{ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::{
    database::{actors::get_actor, establish_connection},
    helpers::api::handle_all_db_errors,
    views::campsites::campsite_view_basic,
    xrpc::{
        auth::Authorization,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/gg.campground.campsite.getActorCampsites", rank = 1)]
pub async fn get_actor_campsites(
    auth: Authorization<'_>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
) -> Result<Json<GetCampsitesOutput>> {
    let actor = get_actor(client, did_document_storage, &auth.actor_did)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let campsite_ids_filtered: Vec<String> =
        actor.campsites.iter().filter_map(|x| x.clone()).collect();

    return get_actor_campsites_from_list(campsite_ids_filtered);
}

#[get("/xrpc/gg.campground.campsite.getActorCampsites?<campsites>", rank = 2)]
pub async fn get_actor_specific_campsites(
    auth: Authorization<'_>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    campsites: Vec<&str>,
) -> Result<Json<GetCampsitesOutput>> {
    let actor = get_actor(client, did_document_storage, &auth.actor_did)
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    let campsite_ids_filtered: Vec<String> = actor
        .campsites
        .iter()
        .filter(|x| {
            x.as_ref()
                .map_or(false, |x| campsites.contains(&x.as_str()))
        })
        .map(|x| x.as_ref().unwrap().clone())
        .collect();

    return get_actor_campsites_from_list(campsite_ids_filtered);
}

fn get_actor_campsites_from_list(campsite_ids: Vec<String>) -> Result<Json<GetCampsitesOutput>> {
    let mut conn = establish_connection().unwrap();

    let campsites = if campsite_ids.is_empty() {
        vec![]
    } else {
        crate::schema::appview::campsite::table
            .filter(crate::schema::appview::campsite::id.eq_any(campsite_ids))
            .load::<Campsite>(&mut conn)
            .map_err(handle_all_db_errors)?
            .iter()
            .map(campsite_view_basic)
            .collect::<Vec<CampsiteViewBasic>>()
    };

    return Ok(Json(GetCampsitesOutput { campsites }));
}
