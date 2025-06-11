use std::str::FromStr;

use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::{Profile, ProfileViewDetailed};
use chrono::DateTime;
use lexicon_cid::CidGeneric;
use rocket::{serde::json::Json,State};
use reqwest::Client;
use rsky_lexicon::com::atproto::repo::Blob;

use crate::{
    database::profiles,
    helpers::views::profile_view_detailed,
    xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.actor.getProfile?<actor>")]
pub async fn get_profile(_auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actor: &str) -> Result<Json<ProfileViewDetailed>> {
    let (actor, db_profile) = profiles::get_profile(client, did_document_storage, actor).await.map_err(|_| XRPCError::NotFound)?;
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
        }
    };

    return Ok(Json(profile_view_detailed(&actor, &record)));
}