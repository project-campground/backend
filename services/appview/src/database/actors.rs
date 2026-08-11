use anyhow::Result;
use appview_schema::schema::appview;
use atproto_identity::model::Document;
use atproto_identity::resolve::resolve_subject;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use diesel::result::Error::NotFound;
use reqwest::Client;

use crate::DNS_RESOLVER;
use crate::database::{establish_connection, models::appview::Actor};
use crate::helpers::api::{handle_all_db_errors, handle_unknown_errors};
use crate::identity::indexing::{DidDocIndexingError, index_actor};
use crate::xrpc::error::XRPCError;
use diesel::prelude::*;

pub fn actor_index_error_to_xrpc_error(err: DidDocIndexingError) -> XRPCError {
    println!("Actor indexing error: {}", err);
    XRPCError::InternalServerError
}

#[allow(unused)]
pub async fn get_actors(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    actors: Vec<String>,
) -> Result<Vec<Actor>, XRPCError> {
    let mut conn = establish_connection().unwrap();

    let mut all_actors = crate::schema::appview::actor::table
        .filter(crate::schema::appview::actor::did.eq_any(&actors))
        .load::<Actor>(&mut conn)
        .map_err(handle_all_db_errors)?;

    let mut actors_to_index = actors.clone();

    for actor in all_actors.iter() {
        let index = actors_to_index.iter().position(|x| x == &actor.did);

        if let Some(index) = index {
            actors_to_index.remove(index);
        }
    }

    let mut newly_indexed_actors: Vec<Actor> =
        futures::future::try_join_all(actors_to_index.iter().map(async |did| {
            let (actor, _) = index_actor(client, did_document_storage, did).await?;

            Ok(actor)
        }))
        .await
        .map_err(actor_index_error_to_xrpc_error)?;

    diesel::insert_into(appview::actor::table)
        .values(&newly_indexed_actors)
        .on_conflict(appview::actor::handle)
        .do_nothing()
        .execute(&mut conn)
        .map_err(handle_all_db_errors)?;

    all_actors.append(&mut newly_indexed_actors);

    Ok(all_actors)
}

pub async fn get_actor(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    actor: &str,
) -> Result<Actor, XRPCError> {
    match get_actor_from_db(actor) {
        Ok(actor) => Ok(actor),
        Err(err) => match err {
            NotFound => {
                let did = resolve_subject(client, &DNS_RESOLVER, actor)
                    .await
                    .map_err(handle_unknown_errors)?;

                let (actor, _) = index_and_save_actor(client, did_document_storage, &did).await?;
                Ok(actor)
            }
            err => Err(handle_all_db_errors(err)),
        },
    }
}

pub async fn index_and_save_actor(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    did: &str,
) -> Result<(Actor, Document), XRPCError> {
    let mut conn = establish_connection().unwrap();

    let (actor, doc) = index_actor(client, did_document_storage, did)
        .await
        .map_err(handle_unknown_errors)?;

    let res = diesel::insert_into(crate::schema::appview::actor::table)
        .values(&actor)
        .on_conflict(appview::actor::handle)
        .do_nothing()
        .returning(Actor::as_returning())
        .get_result(&mut conn)
        .map_err(handle_all_db_errors)?;

    Ok((res, doc))
}

pub fn get_actor_from_db(actor: &str) -> Result<Actor, diesel::result::Error> {
    let mut conn = establish_connection().unwrap();

    crate::schema::appview::actor::table
        .filter(
            crate::schema::appview::actor::did
                .eq(actor)
                .or(crate::schema::appview::actor::handle.eq(actor)),
        )
        .first::<Actor>(&mut conn)
}
