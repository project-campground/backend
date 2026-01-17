use anyhow::Result;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use atproto_identity::{plc, web};
use atproto_identity::resolve::resolve_subject;
use campground_lexicon::gg::campground::home_server::HomeServer;
use common::fetch_record;
use diesel::result::Error::NotFound;
use reqwest::Client;

use diesel::prelude::*;
use crate::DNS_RESOLVER;
use crate::database::{
    establish_connection,
    models::appview::Actor,
};

pub async fn get_actors(client: &Client, did_document_storage: &LruDidDocumentStorage, actors: Vec<String>) -> Result<Vec<Actor>> {
    let mut conn = establish_connection().unwrap();

    let mut db_actors = Vec::new();
    let query_result = crate::schema::appview::actor::table
        .filter(crate::schema::appview::actor::did.eq_any(&actors))
        .load::<Actor>(&mut conn)
        .expect("Error loading actors");
    let mut to_insert = Vec::new();

    for actor in actors {
        match query_result.iter().find(|a| a.did == actor) {
            Some(actor) => db_actors.push(actor.clone()),
            None => {
                let actor = resolve_subject(client, &DNS_RESOLVER, &actor).await.ok();
                if let Some(actor) = actor {
                    let doc = match *actor.split(":").collect::<Vec<&str>>().get(1).unwrap() {
                        "plc" => {
                            plc::query(client, "plc.directory", &actor).await?
                        },
                        "web" => {
                            web::query(client, &actor).await?
                        }
                        _ => {
                            continue;
                        }
                    };
                
                    let handle = doc.also_known_as
                        .iter()
                        .find(|&handle| handle.starts_with("at://"));
                
                    let home_server = fetch_record::<HomeServer>(&actor, "gg.campground.homeServer", "self", client, did_document_storage, &DNS_RESOLVER).await?;
                    
                    if home_server.value.did != "" && home_server.value.did.starts_with("did:") {
                        match handle {
                            Some(handle) => {
                                // Validate that the handle resolves back to the DID
                                let resolved = resolve_subject(client, &DNS_RESOLVER, handle).await?;
                                if resolved == actor {
                                    let actor = Actor {
                                        did: actor.clone(),
                                        handle: Some(handle.clone()),
                                        home_server: home_server.value.did.clone(),
                                        indexed_at: chrono::Utc::now().naive_utc().to_string(),
                                        campsites: vec![],
                                    };
                                    to_insert.push(actor.clone());
                                    db_actors.push(actor);
                                }
                            },
                            None => {
                                let actor = Actor {
                                    did: actor.clone(),
                                    handle: None,
                                    home_server: home_server.value.did.clone(),
                                    indexed_at: chrono::Utc::now().naive_utc().to_string(),
                                    campsites: vec![],
                                };
                                to_insert.push(actor.clone());
                                db_actors.push(actor);
                            }
                        }
                    }
                }
            }
        }
    }

    if !to_insert.is_empty() {
        diesel::insert_into(crate::schema::appview::actor::table)
            .values(&to_insert)
            .execute(&mut conn)
            .expect("Error saving new actors");
    }

    Ok(db_actors)
}

pub async fn get_actor(client: &Client, did_document_storage: &LruDidDocumentStorage, actor: &str) -> Result<Actor> {
    let mut conn = establish_connection().unwrap();

    if actor.starts_with("did:") {
        let db_actor = crate::schema::appview::actor::table
            .filter(crate::schema::appview::actor::did.eq(actor))
            .first::<Actor>(&mut conn);

        match db_actor {
            Ok(actor) => Ok(actor),
            Err(err) => {
                match err {
                    NotFound => {
                        index_actor(client, did_document_storage, actor).await?;
                        let actor = crate::schema::appview::actor::table
                            .filter(crate::schema::appview::actor::did.eq(actor))
                            .first::<Actor>(&mut conn)
                            .expect("Error fetching actor");
                        Ok(actor)
                    },
                    _ => Err(err.into())
                }
            }
        }
    } else {
        let db_actor = crate::schema::appview::actor::table
            .filter(crate::schema::appview::actor::handle.eq(format!("at://{}", actor)))
            .first::<Actor>(&mut conn);

        match db_actor {
            Ok(actor) => Ok(actor),
            Err(err) => {
                match err {
                    NotFound => {
                        index_actor(client, did_document_storage, actor).await?;
                        let actor = crate::schema::appview::actor::table
                            .filter(crate::schema::appview::actor::handle.eq(actor))
                            .first::<Actor>(&mut conn)
                            .expect("Error fetching actor");
                        Ok(actor)
                    },
                    _ => Err(err.into())
                }
            }
        }
    }
}

pub async fn index_actor(client: &Client, did_document_storage: &LruDidDocumentStorage, actor: &str) -> Result<()> {
    let did = resolve_subject(client, &DNS_RESOLVER, actor).await?;
    let mut conn = establish_connection().unwrap();

    let doc = match *did.split(":").collect::<Vec<&str>>().get(1).unwrap() {
        "plc" => {
            plc::query(client, "plc.directory", &did).await?
        },
        "web" => {
            web::query(client, &did).await?
        }
        _ => {
            return Ok(())
        }
    };

    let handle = doc.also_known_as
        .iter()
        .find(|&handle| handle.starts_with("at://"));

    let home_server = fetch_record::<HomeServer>(actor, "gg.campground.homeServer", "self", client, did_document_storage, &DNS_RESOLVER).await?;

    if home_server.value.did == "" || !home_server.value.did.starts_with("did:") {
        // Only index actors with a valid Campground home server
        return Ok(())
    }

    match handle {
        Some(handle) => {            
            // Validate that the handle resolves back to the DID
            let resolved = resolve_subject(client, &DNS_RESOLVER, handle).await?;
            if resolved != did {
                return Ok(())
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
                campsites: vec![]
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