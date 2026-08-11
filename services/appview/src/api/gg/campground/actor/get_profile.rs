use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::ProfileViewDetailed;
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::{
    database::profiles,
    views::profiles::profile_view_detailed,
    xrpc::error::{Result, XRPCError},
};

// Gets Campground profile record, not Bluesky's profile. This means that you can have totally separate Bluesky and Campground profiles.
// FIXME Shall this be changed? Perhaps make it an optional record that could overwrite existing Bluesky profile, but fallback to Bluesky profile if record does not exist?
#[get("/xrpc/gg.campground.actor.getProfile?<actor>")]
pub async fn get_profile(
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    actor: &str,
) -> Result<Json<ProfileViewDetailed>> {
    let (actor, profile) = profiles::get_profile(client, did_document_storage, actor).await?;

    if let Some(profile) = profile {
        Ok(Json(profile_view_detailed(&actor, Some(&profile))))
    } else {
        Err(XRPCError::NotFound)
    }
}
