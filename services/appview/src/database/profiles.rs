use anyhow::Result;
use appview_schema::models::appview::Actor;
use appview_schema::schema::appview;
use atproto_client::client::Auth;
use atproto_client::com::atproto::repo::{GetRecordResponse, get_record};
use atproto_identity::model::Document;
use atproto_identity::resolve::resolve_subject;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::Profile as ProfileRecord;
use chrono::Utc;
use reqwest::Client;
use rsky_lexicon::com::atproto::repo::Blob;
use serde_json::from_value;

use crate::DNS_RESOLVER;
use crate::database::actors::{actor_index_error_to_xrpc_error, index_and_save_actor};
use crate::database::{establish_connection, models::appview::Profile};
use crate::helpers::api::{handle_all_db_errors, handle_unknown_errors};
use crate::identity::indexing::{get_actor_did_doc, index_actor};
use crate::util::string::AppviewStringExtension;
use crate::xrpc::error::XRPCError;
use diesel::prelude::*;

pub async fn get_profile_from_actor(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    actor: Actor,
) -> Result<(Actor, Option<Profile>), XRPCError> {
    let mut conn = establish_connection().unwrap();

    let profile = appview::profile::table
        .filter(appview::profile::creator.eq(&actor.did))
        .first::<Profile>(&mut conn);

    match profile {
        Ok(profile) => Ok((actor, Some(profile))),
        Err(diesel::result::Error::NotFound) => {
            index_profile_from_actor(client, did_document_storage, actor).await
        }
        Err(err) => Err(handle_all_db_errors(err)),
    }
}

pub async fn get_profile(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    actor: &str,
) -> Result<(Actor, Option<Profile>), XRPCError> {
    match get_profile_and_actor_from_db(actor) {
        Ok((actor, profile)) => match profile {
            Some(profile) => Ok((actor, Some(profile))),
            None => index_profile_from_actor(client, did_document_storage, actor).await,
        },
        Err(err) => match err {
            diesel::result::Error::NotFound => {
                let did = resolve_subject(client, &DNS_RESOLVER, actor)
                    .await
                    .map_err(handle_unknown_errors)?;

                let (actor, actor_did_doc) =
                    index_and_save_actor(client, did_document_storage, &did).await?;

                index_and_save_profile(client, &actor_did_doc, actor).await
            }
            err => Err(handle_all_db_errors(err)),
        },
    }
}
pub async fn index_profile_from_actor(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    actor: Actor,
) -> Result<(Actor, Option<Profile>), XRPCError> {
    let actor_did_doc = get_actor_did_doc(client, did_document_storage, &actor.did)
        .await
        .map_err(actor_index_error_to_xrpc_error)?;

    index_and_save_profile(client, &actor_did_doc, actor).await
}
pub async fn get_profiles(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    did_list: Vec<String>,
) -> Result<Vec<(Actor, Option<Profile>)>, XRPCError> {
    let mut conn = establish_connection().unwrap();

    let mut actors_profiles = appview::actor::table
        .filter(appview::actor::did.eq_any(&did_list))
        .left_join(appview::profile::table.on(appview::profile::creator.eq(appview::actor::did)))
        .load::<(Actor, Option<Profile>)>(&mut conn)
        .map_err(handle_all_db_errors)?;

    let mut unindexed_actors = did_list.clone();

    // So there are less actors missing
    for (actor, _) in actors_profiles.iter() {
        let index = unindexed_actors.iter().position(|x| x == &actor.did);

        if let Some(index) = index {
            unindexed_actors.remove(index);
        }
    }

    let mut newly_indexed_actors = futures::future::try_join_all(
        unindexed_actors
            .iter()
            .map(|did| index_actor(client, did_document_storage, did)),
    )
    .await
    .map_err(actor_index_error_to_xrpc_error)?;

    // Make sure we save new ones for next times. We just indexed and did nothing else
    let db_actors: Vec<Actor> = newly_indexed_actors
        .iter()
        .map(|(actor, _)| actor.clone())
        .collect();

    diesel::insert_into(appview::actor::table)
        .values(db_actors)
        .on_conflict(appview::actor::handle)
        .do_nothing()
        .execute(&mut conn)
        .map_err(handle_all_db_errors)?;

    // To supply info when known immediately, not having users to manually index profile when it appears
    let mut existing_actors_without_profiles: Vec<(Actor, Document)> =
        futures::future::try_join_all(
            actors_profiles
                .iter()
                .filter(|(_, profile)| profile.is_none())
                .map(async |(actor, _)| {
                    let doc = get_actor_did_doc(client, did_document_storage, &actor.did).await?;
                    Ok((actor.clone(), doc))
                }),
        )
        .await
        .map_err(actor_index_error_to_xrpc_error)?;

    // To get profiles of everything in one swing
    existing_actors_without_profiles.append(&mut newly_indexed_actors);

    let mut newly_indexed_profiles: Vec<(Actor, Option<Profile>)> =
        futures::future::try_join_all(existing_actors_without_profiles.iter().map(
            async |(actor, doc)| {
                let (actor, indexed_profile) = index_profile(client, doc, actor.clone()).await?;

                // Since the profile can still be none
                if let Some((cid, profile_record)) = indexed_profile {
                    let profile = profile_record_to_db_profile(&cid, &actor, &profile_record);
                    Ok((actor, Some(profile)))
                } else {
                    Ok((actor, None))
                }
            },
        ))
        .await?;

    let profiles_to_insert: Vec<&Profile> = newly_indexed_profiles
        .iter()
        .filter_map(|(_, profile)| profile.as_ref())
        .collect();

    diesel::insert_into(appview::profile::table)
        .values(profiles_to_insert)
        .execute(&mut conn)
        .map_err(handle_all_db_errors)?;

    actors_profiles.append(&mut newly_indexed_profiles);

    Ok(actors_profiles)
}

fn get_blob_ref(blob: Option<&Blob>) -> Option<String> {
    blob.map(|blob| blob.r#ref.map(|r| r.to_string())).flatten()
}
pub fn profile_record_to_db_profile(
    cid: &str,
    actor: &Actor,
    profile_record: &ProfileRecord,
) -> Profile {
    Profile {
        cid: cid.to_string(),
        creator: actor.did.clone(),
        uri: format!("at://{}/gg.campground.actor.profile/self", &actor.did),
        display_name: profile_record
            .display_name
            .clone()
            .map(|val| val.truncate_clone(64)),
        description: profile_record
            .description
            .clone()
            .map(|val| val.truncate_clone(256)),
        avatar_cid: get_blob_ref(profile_record.avatar.as_ref()),
        banner_cid: get_blob_ref(profile_record.banner.as_ref()),
        indexed_at: chrono::Utc::now().naive_utc().to_string(),
        created_at: match profile_record.created_at {
            Some(created_at) => Some(created_at.to_string()),
            None => None,
        },
        tagline: profile_record
            .tagline
            .clone()
            .map(|val| val.truncate_clone(128)),
        location: profile_record
            .location
            .clone()
            .map(|val| val.truncate_clone(128)),
        first_seen: Utc::now().to_string(),
    }
}

pub fn get_pds_from_did_doc(did_document: &Document) -> Option<String> {
    let pds_service = did_document.service.iter().find(|service| {
        service.id == "#atproto_pds" && service.r#type == "AtprotoPersonalDataServer"
    })?;

    Some(pds_service.service_endpoint.clone())
}

pub async fn index_profile(
    client: &Client,
    actor_did_doc: &Document,
    actor: Actor,
) -> Result<(Actor, Option<(String, ProfileRecord)>), XRPCError> {
    let pds = get_pds_from_did_doc(actor_did_doc);
    println!("Pds: {:?}", pds);

    if pds.is_none() {
        return Ok((actor, None));
    };

    let pds = &pds.unwrap();
    let profile = get_profile_from_pds(client, &pds, &actor.did)
        .await
        .map_err(handle_unknown_errors)?;

    match profile {
        GetRecordResponse::Error(_) => Ok((actor, None)),
        GetRecordResponse::Record {
            uri: _,
            cid,
            value,
            extra: _,
        } => {
            let profile_record =
                from_value::<ProfileRecord>(value).map_err(handle_unknown_errors)?;
            Ok((actor, Some((cid, profile_record))))
        }
    }
}

pub async fn index_and_save_profile(
    client: &Client,
    actor_did_doc: &Document,
    actor: Actor,
) -> Result<(Actor, Option<Profile>), XRPCError> {
    let (actor, profile_record) = index_profile(client, actor_did_doc, actor).await?;

    match profile_record {
        None => Ok((actor, None)),
        Some((cid, profile_record)) => {
            let db_profile = profile_record_to_db_profile(&cid, &actor, &profile_record);

            let mut conn = establish_connection().unwrap();

            diesel::insert_into(appview::profile::table)
                .values(&db_profile)
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .map_err(handle_all_db_errors)?;

            Ok((actor, Some(db_profile)))
        }
    }
}

pub fn get_profile_and_actor_from_db(
    actor: &str,
) -> Result<(Actor, Option<Profile>), diesel::result::Error> {
    let mut conn = establish_connection().unwrap();

    appview::actor::table
        .filter(
            appview::actor::did
                .eq(actor)
                .or(appview::actor::handle.eq(actor)),
        )
        .left_join(appview::profile::table.on(appview::profile::creator.eq(appview::actor::did)))
        .first::<(Actor, Option<Profile>)>(&mut conn)
}

pub async fn get_profile_from_pds(
    client: &Client,
    pds_domain: &str,
    did: &str,
) -> Result<GetRecordResponse, anyhow::Error> {
    get_record(
        client,
        &Auth::None,
        pds_domain,
        did,
        "gg.campground.actor.profile",
        "self",
        None,
    )
    .await
}
