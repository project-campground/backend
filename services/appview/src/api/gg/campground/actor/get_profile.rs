use std::str::FromStr;

use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::{Profile, ProfileViewDetailed};
use chrono::DateTime;
use lexicon_cid::CidGeneric;
use reqwest::Client;
use rocket::{State, serde::json::Json};
use rsky_lexicon::com::atproto::repo::Blob;

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
    let (actor, db_profile) = profiles::get_existing_profile(client, did_document_storage, actor)
        .await
        .map_err(|_| XRPCError::NotFound)?;
    let record = Profile {
        display_name: db_profile.display_name,
        description: db_profile.description,
        avatar: match db_profile.avatar_cid {
            Some(cid) => Some(Blob {
                r#type: None,
                r#ref: Some(CidGeneric::from_str(&cid).unwrap()),
                cid: None,
                mime_type: "image".to_string(),
                size: None,
                original: None,
            }),
            None => None,
        },
        banner: match db_profile.banner_cid {
            Some(cid) => Some(Blob {
                r#type: None,
                r#ref: Some(CidGeneric::from_str(&cid).unwrap()),
                cid: None,
                mime_type: "image".to_string(),
                size: None,
                original: None,
            }),
            None => None,
        },
        tagline: db_profile.tagline,
        location: db_profile.location,
        social_connections: None,
        labels: None,
        created_at: match db_profile.created_at {
            Some(datetime) => DateTime::from_str(&datetime).ok(),
            None => None,
        },
    };

    return Ok(Json(profile_view_detailed(&actor, &record)));
}
