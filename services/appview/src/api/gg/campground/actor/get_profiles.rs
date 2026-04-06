use std::str::FromStr;

use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::{GetProfilesOutput, Profile, ProfileViewDetailed};
use chrono::DateTime;
use lexicon_cid::CidGeneric;
use reqwest::Client;
use rocket::{State, serde::json::Json};
use rsky_lexicon::com::atproto::repo::Blob;

use crate::{
    database::profiles,
    helpers::{deduplicate_list, lower_list},
    views::profiles::profile_view_detailed,
    xrpc::error::{Result, XRPCError},
};

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
    let mut profile_views: Vec<ProfileViewDetailed> = vec![];
    let db_profiles = profiles::get_profiles(client, did_document_storage, actors)
        .await
        .map_err(|_| XRPCError::InternalServerError)?;
    for (actor, db_profile) in db_profiles {
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
        profile_views.push(profile_view_detailed(&actor, &record));
    }

    if profile_views.len() == 0 {
        return Err(XRPCError::BadRequest("".to_string()));
    }

    return Ok(Json(GetProfilesOutput {
        profiles: profile_views,
    }));
}
