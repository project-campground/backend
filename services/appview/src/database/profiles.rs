use anyhow::Result;
use appview_schema::models::appview::Actor;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::Profile as ProfileRecord;
use chrono::Utc;
use common::{fetch_record, get_blob_ref};
use diesel::result::Error::NotFound;
use reqwest::Client;

use diesel::prelude::*;
use crate::database::actors::get_actor;
use crate::DNS_RESOLVER;
use crate::database::{
    establish_connection,
    models::appview::Profile,
};

pub async fn get_profiles(client: &Client, did_document_storage: &LruDidDocumentStorage, actors: Vec<String>) -> Result<Vec<(Actor, Profile)>> {
    let mut conn = establish_connection().unwrap();

    let db_actors = crate::database::actors::get_actors(client, did_document_storage, actors).await?;
    let actors: Vec<&str> = db_actors.iter().map(|actor| actor.did.as_str()).collect();

    let mut db_profiles = Vec::new();
    let query_result = crate::schema::appview::profile::table
        .filter(crate::schema::appview::profile::creator.eq_any(&actors))
        .load::<Profile>(&mut conn)
        .expect("Failed to query profiles");
    let mut to_insert = Vec::new();

    for actor in actors {
        match query_result.iter().find(|profile| profile.creator == actor) {
            Some(profile) => db_profiles.push((db_actors.iter().find(|a| a.did == actor).unwrap().to_owned(), profile.clone())),
            None => {
                let profile_record = fetch_record::<ProfileRecord>(
                    actor,
                    "gg.campground.actor.profile",
                    "self",
                    client,
                    did_document_storage,
                    &DNS_RESOLVER
                ).await?;
                let profile = Profile {
                    cid: profile_record.cid.to_string(),
                    uri: format!("at://{}/gg.campground.actor.profile/self", actor),
                    creator: actor.to_string(),
                    display_name: profile_record.value.display_name,
                    description: profile_record.value.description,
                    avatar_cid: get_blob_ref(&profile_record.value.avatar),
                    banner_cid: get_blob_ref(&profile_record.value.banner),
                    indexed_at: chrono::Utc::now().naive_utc().to_string(),
                    created_at: match profile_record.value.created_at {
                        Some(created_at) => Some(created_at.to_string()),
                        None => None
                    },
                    tagline: profile_record.value.tagline,
                    location: profile_record.value.location,
                    first_seen: Utc::now().to_string(),
                };
                to_insert.push(profile.clone());
                db_profiles.push((db_actors.iter().find(|a| a.did == actor).unwrap().to_owned(), profile));
            }
        }
    }
    if !to_insert.is_empty() {
        diesel::insert_into(crate::schema::appview::profile::table)
            .values(&to_insert)
            .execute(&mut conn)?;
    }
    Ok(db_profiles)
}

pub async fn get_profile(client: &Client, did_document_storage: &LruDidDocumentStorage, actor: &str) -> Result<(Actor, Profile)> {
    let actor = get_actor(client, did_document_storage, actor).await?;

    get_profile_from_actor(client, did_document_storage, actor).await
}
pub async fn get_profile_from_actor(client: &Client, did_document_storage: &LruDidDocumentStorage, actor: Actor) -> Result<(Actor, Profile)> {
    let mut conn = establish_connection().unwrap();
    let db_profile = crate::schema::appview::profile::table
        .filter(crate::schema::appview::profile::creator.eq(&actor.did))
        .first::<Profile>(&mut conn);

    match db_profile {
        Ok(profile) => Ok((actor, profile)),
        Err(NotFound) => {
            let profile_record = fetch_record::<ProfileRecord>(
                &actor.did,
                "gg.campground.actor.profile",
                "self",
                client,
                did_document_storage,
                &DNS_RESOLVER
            ).await?;

            let profile = Profile {
                cid: profile_record.cid.to_string(),
                uri: format!("at://{}/gg.campground.actor.profile/self", &actor.did),
                creator: actor.did.clone(),
                display_name: profile_record.value.display_name,
                description: profile_record.value.description,
                avatar_cid: get_blob_ref(&profile_record.value.avatar),
                banner_cid: get_blob_ref(&profile_record.value.banner),
                indexed_at: chrono::Utc::now().naive_utc().to_string(),
                created_at: match profile_record.value.created_at {
                    Some(created_at) => Some(created_at.to_string()),
                    None => None
                },
                tagline: profile_record.value.tagline,
                location: profile_record.value.location,
                first_seen: Utc::now().to_string(),
            };

            let mut conn = establish_connection().unwrap();
            diesel::insert_into(crate::schema::appview::profile::table)
                .values(&profile)
                .execute(&mut conn)?;
            Ok((actor, profile))
        },
        Err(e) => Err(e.into()),
    }
}