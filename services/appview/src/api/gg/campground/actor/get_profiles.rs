use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::GetProfilesOutput;
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::{
    helpers::{deduplicate_list, lower_list},
    views::profiles::profile_view_basic,
    xrpc::error::{Result, XRPCError},
};

// Gets multiple Campground profiles (not Bluesky profiles)
#[get("/xrpc/gg.campground.actor.getProfiles?<actors>")]
pub async fn get_profiles(
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    actors: Vec<&str>,
) -> Result<Json<GetProfilesOutput>> {
    let actors = deduplicate_list(lower_list(actors));
    if actors.len() > 25 || actors.len() == 0 {
        return Err(XRPCError::BadRequest(
            "actors query must have at least 1 actor and less than or equal to 25".to_string(),
        ));
    }

    let profiles = crate::database::profiles::get_profiles(
        client,
        did_document_storage,
        actors.iter().map(|x| x.to_string()).collect(),
    )
    .await?
    .iter()
    .map(|(actor, profile)| profile_view_basic(&actor, profile.as_ref()))
    .collect();

    return Ok(Json(GetProfilesOutput { profiles }));
}
