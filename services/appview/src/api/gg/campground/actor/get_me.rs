use appview_schema::models::appview::Campsite;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::{actor::GetMeOutput, campsite::CampsiteViewBasic};
use diesel::{ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::{
    database::{establish_connection, profiles},
    helpers::api::handle_all_db_errors,
    views::{
        campsites::campsite_view_basic,
        profiles::profile_view_basic_or_empty,
    },
    xrpc::{
        auth::Authorization,
        error::{Result, XRPCError},
    },
};

// TODO: Remove this without deprecation before release
#[get("/xrpc/gg.campground.actor.getMe")]
pub async fn get_me(
    auth: Authorization<'_>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
) -> Result<Json<GetMeOutput>> {
    let mut conn = establish_connection().unwrap();
    let (actor, db_profile) =
        profiles::get_profile(client, did_document_storage, auth.actor_did.as_str())
            .await
            .map_err(|_| XRPCError::Unauthorized)?;

    let campsite_ids_filtered: Vec<String> =
        actor.campsites.iter().filter_map(|x| x.clone()).collect();

    let campsites = if actor.campsites.is_empty() {
        vec![]
    } else {
        crate::schema::appview::campsite::table
            .filter(crate::schema::appview::campsite::id.eq_any(campsite_ids_filtered))
            .load::<Campsite>(&mut conn)
            .map_err(handle_all_db_errors)?
            .iter()
            .map(campsite_view_basic)
            .collect::<Vec<CampsiteViewBasic>>()
    };

    return Ok(Json(GetMeOutput {
        campsites,
        profile: profile_view_basic_or_empty(&actor, db_profile.as_ref()),
    }));
}
