#![deny(unsafe_code)]
#![warn(
    clippy::cognitive_complexity,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::float_equality_without_abs,
    keyword_idents,
    clippy::missing_const_for_fn,
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::mod_module_files,
    non_ascii_idents,
    noop_method_call,
    clippy::option_if_let_else,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::semicolon_if_nothing_returned,
    clippy::unseparated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::suspicious_operation_groupings,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    clippy::unused_self,
    clippy::use_debug,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq,
    clippy::wildcard_dependencies,
    clippy::wildcard_imports
)]

use std::{env, num::NonZero, str::FromStr, sync::Arc, time::Duration};

use anyhow::Result;
use appview_schema::models::appview::Setting;
use atproto_identity::{resolve::{create_resolver, resolve_subject}, storage_lru::LruDidDocumentStorage};
use crate::jetstream::*;
use campground_lexicon::gg::campground::{actor::Profile, home_server::HomeServer};
use chrono::Utc;
use common::{ResponseResult, fetch_record};
use diesel::prelude::*;
use hickory_resolver::TokioResolver;
use lazy_static::lazy_static;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures::StreamExt as _;
use url::Url;

use crate::database::{establish_connection, models::appview::Actor};

#[macro_use] extern crate serde;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
    static ref DNS_RESOLVER: Arc<TokioResolver> = Arc::new(create_resolver(&[]));
    static ref DID_DOCUMENT_STORAGE: LruDidDocumentStorage = LruDidDocumentStorage::new(NonZero::new(6000).unwrap());
}

async fn discover_actor(actor: &str, force: Option<bool>) -> Result<()> {
    let did = resolve_subject(&HTTP_CLIENT, &DNS_RESOLVER, actor).await?;
    let mut conn = establish_connection().unwrap();

    if !force.unwrap_or(false) {
        let existing_actor = crate::schema::appview::actor::table
            .filter(crate::schema::appview::actor::handle.eq(actor))
            .get_result::<Actor>(&mut conn)
            .optional()
            .expect("Error loading actor");

        if let Some(existing_actor) = existing_actor {
            if existing_actor.did == did {
                let indexed_at: chrono::DateTime<Utc> = chrono::DateTime::from_str(&existing_actor.indexed_at)
                    .expect("Error parsing indexed_at");
                if indexed_at > Utc::now() - chrono::Duration::days(1) {
                    // Only re-index an actor if the last index happened longer than a day ago
                    return Ok(())
                }
            }
        }
    }
    
    let doc = common::fetch_did_document(actor, &DNS_RESOLVER, &DID_DOCUMENT_STORAGE, &HTTP_CLIENT).await?;
    let handle = doc.also_known_as
        .iter()
        .find(|&handle| handle.starts_with("at://"));

    let home_server = fetch_record::<HomeServer>(actor, "gg.campground.homeServer", "self", &HTTP_CLIENT, &DID_DOCUMENT_STORAGE, &DNS_RESOLVER).await?;

    if let ResponseResult::Error { error: _, message: _ } = home_server {
        return Ok(());
    }

    let home_server = home_server.unwrap();

    if home_server.value.did == "" || !home_server.value.did.starts_with("did:") {
        // Only index actors with a valid Campground home server
        return Ok(())
    }

    match handle {
        Some(handle) => {            
            // Validate that the handle resolves back to the DID
            let resolved = resolve_subject(&HTTP_CLIENT, &DNS_RESOLVER, handle).await?;
            if resolved != did {
                return Ok(());
            }

            let actor = Actor {
                did: did.clone(),
                handle: Some(handle.clone()),
                home_server: home_server.value.did.clone(),
                indexed_at: chrono::Utc::now().naive_utc().to_string(),
                campsites: vec![],
            };
            diesel::insert_into(crate::schema::appview::actor::table)
                .values(&actor)
                .on_conflict(crate::schema::appview::actor::handle)
                .do_update()
                .set(&actor)
                .returning(Actor::as_returning())
                .get_result(&mut conn)
                .expect("Error updating actor");
            Ok(())
        },
        None => {
            let actor = Actor {
                did: did.clone(),
                handle: None,
                home_server: home_server.value.did.clone(),
                indexed_at: chrono::Utc::now().naive_utc().to_string(),
                campsites: vec![],
            };
            diesel::insert_into(crate::schema::appview::actor::table)
                .values(&actor)
                .on_conflict(crate::schema::appview::actor::handle)
                .do_update()
                .set(&actor)
                .returning(Actor::as_returning())
                .get_result(&mut conn)
                .expect("Error updating actor");
            Ok(())
        }
    }
}

async fn process(message: String) {
    match read(&message) {
        Ok(body) => {
            match body {
                JetstreamRepoMessage::Commit(commit) => {
                    // Update the cursor every 20~ events
                    if commit.time_us.rem_euclid(20) == 0 {
                        let setting = Setting {
                            name: "cursor".to_string(),
                            value: Some(commit.time_us.to_string())
                        };
                        let mut conn = establish_connection().unwrap();
                        diesel::insert_into(crate::schema::appview::setting::table)
                            .values(&setting)
                            .on_conflict(crate::schema::appview::setting::name)
                            .do_update()
                            .set(&setting)
                            .returning(Setting::as_returning())
                            .get_result(&mut conn)
                            .expect("Error updating setting");
                    }

                    match commit.commit.collection.as_str() {
                        "gg.campground.actor.profile" => {
                            match commit.commit.operation.as_str() {
                                "create" | "update" => {
                                    if !commit.commit.record.is_some() {
                                        return;
                                    }
                                    let profile: Result<Profile, _> = serde_json::from_value(commit.commit.record.unwrap());
                                    match profile {
                                        Ok(record) => {
                                            use crate::database::models::appview::Profile;

                                            let _ = discover_actor(&commit.did, None).await;
                                            if !commit.commit.cid.is_some() {
                                                return;
                                            }

                                            let mut first_seen = Utc::now().to_string();
                                            let mut conn = establish_connection().unwrap();
                                            
                                            let existing = crate::schema::appview::profile::table
                                                .filter(crate::schema::appview::profile::creator.eq(&commit.did))
                                                .first::<Profile>(&mut conn)
                                                .optional()
                                                .expect("Error loading profile");
                                            if let Some(existing) = existing {
                                                first_seen = existing.first_seen;
                                            }
                                            
                                            let profile = Profile {
                                                uri: format!("at://{}/{}/{}", commit.did, commit.commit.collection, commit.commit.rkey),
                                                cid: commit.commit.cid.unwrap(),
                                                creator: commit.did,
                                                display_name: record.display_name,
                                                description: record.description,
                                                avatar_cid: match record.avatar {
                                                    Some(avatar) => Some(avatar.cid.unwrap()),
                                                    None => None
                                                },
                                                banner_cid: match record.banner {
                                                    Some(banner) => Some(banner.cid.unwrap()),
                                                    None => None
                                                },
                                                indexed_at: chrono::Utc::now().naive_utc().to_string(),
                                                tagline: record.tagline,
                                                created_at: match record.created_at {
                                                    Some(created_at) => Some(created_at.to_string()),
                                                    None => None
                                                },
                                                location: record.location,
                                                first_seen: first_seen
                                            };
                                            diesel::insert_into(crate::schema::appview::profile::table)
                                                .values(&profile)
                                                .on_conflict(crate::schema::appview::profile::cid)
                                                .do_update()
                                                .set(&profile)
                                                .returning(Profile::as_returning())
                                                .get_result(&mut conn)
                                                .expect("Error updating profile");
                                        },
                                        _ => ()
                                    }
                                },
                                "delete" => {
                                    let mut conn = establish_connection().unwrap();
                                    diesel::delete(crate::schema::appview::profile::table)
                                        .filter(crate::schema::appview::profile::uri.eq(format!("at://{}/{}/{}", &commit.did, commit.commit.collection, commit.commit.rkey)))
                                        .filter(crate::schema::appview::profile::creator.eq(&commit.did))
                                        .execute(&mut conn)
                                        .expect("Error deleting profile");

                                    diesel::delete(crate::schema::appview::actor::table)
                                        .filter(crate::schema::appview::actor::did.eq(&commit.did))
                                        .execute(&mut conn)
                                        .expect("Error deleting actor");
                                },
                                _ => ()
                            }
                        },
                        "gg.campground.homeServer" => {
                            match commit.commit.operation.as_str() {
                                "create" | "update" => {
                                    let _ = discover_actor(&commit.did, Some(true)).await;
                                },
                                "delete" => {
                                    let mut conn = establish_connection().unwrap();
                                    diesel::delete(crate::schema::appview::actor::table)
                                        .filter(crate::schema::appview::actor::did.eq(&commit.did))
                                        .execute(&mut conn)
                                        .expect("Error deleting actor");
                                    diesel::delete(crate::schema::appview::profile::table)
                                        .filter(crate::schema::appview::profile::creator.eq(&commit.did))
                                        .execute(&mut conn)
                                        .expect("Error deleting profile");
                                },
                                _ => ()
                            }
                        }
                        _ => {}
                    }
                },
                JetstreamRepoMessage::Identity(identity) => {
                    let _ = discover_actor(&identity.did, Some(true)).await;
                },
                JetstreamRepoMessage::Account(_account) => {
                    // Do nothing for now
                }
            }
        }
        Err(_) => ()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    let default_subscriber_path = env::var("JETSTREAM_SERVER_ENDPOINT")
        .unwrap_or("wss://jetstream1.us-west.bsky.network".into());
    let wanted_collections = vec!["gg.campground.actor.profile", "gg.campground.homeServer"];
    let mut cursor = "".to_string();

    let mut conn = establish_connection().unwrap();
    let setting = crate::schema::appview::setting::table
        .filter(crate::schema::appview::setting::name.eq("cursor"))
        .get_result::<Setting>(&mut conn)
        .optional()
        .expect("Error loading setting");
    if setting.is_some() {
        cursor = format!("&cursor={}", setting.unwrap().value.unwrap());
    }

    loop {
        match tokio_tungstenite::connect_async(
            Url::parse(
                format!(
                    "{sub}/subscribe?{filter}{cursor}",
                    sub = default_subscriber_path,
                    filter = wanted_collections
                        .iter()
                        .map(|c| format!("wantedCollections={}", c))
                        .collect::<Vec<String>>()
                        .join("&"),
                    cursor = cursor
                ).as_str()
            )
            .unwrap(),
        )
        .await
        {
            Ok((mut socket, _response)) => {
                while let Some(Ok(Message::Text(message))) = socket.next().await {
                    tokio::spawn(async move {
                        process(message).await;
                    });
                }
            }
            Err(e) => {
                println!("Error connecting to jetstream: {}", e);
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
        }
    }
}

mod jetstream;
mod database;
pub use appview_schema::schema;