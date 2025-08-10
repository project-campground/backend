use anyhow::Result;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use reqwest::Client;

use diesel::prelude::*;
use crate::database::actors::get_actor;
use crate::database::{
    establish_connection,
    models::appview::{Account, Block, Friend},
};

pub async fn get_accounts(client: &Client, did_document_storage: &LruDidDocumentStorage, actors: Vec<String>) -> Result<Vec<Account>> {
    let mut conn = establish_connection().unwrap();

    let db_actors = crate::database::actors::get_actors(client, did_document_storage, actors).await?;
    let actors: Vec<&str> = db_actors.iter().map(|actor| actor.did.as_str()).collect();
    
    let mut db_accounts = Vec::new();
    let query_result = crate::schema::appview::account::table
        .filter(crate::schema::appview::account::did.eq_any(&actors))
        .load::<Account>(&mut conn)
        .expect("Error loading accounts");

    for actor in actors {
        let account = query_result.iter().find(|account| account.did == actor);
        if let Some(account) = account {
            db_accounts.push(account.clone());
        }
    }
    Ok(db_accounts)
}

pub async fn get_account(client: &Client, did_document_storage: &LruDidDocumentStorage, actor: &str) -> Result<Account> {
    let actor = get_actor(client, did_document_storage, actor).await?;

    let mut conn = establish_connection().unwrap();
    let db_account = crate::schema::appview::account::table
        .filter(crate::schema::appview::account::did.eq(&actor.did))
        .first::<Account>(&mut conn)?;

    Ok(db_account)
}

pub async fn are_actors_friends(client: &Client, did_document_storage: &LruDidDocumentStorage, actor_a: &str, actor_b: &str) -> Result<bool> {
    let actor_a = get_actor(client, did_document_storage, actor_a).await?;
    let actor_b = get_actor(client, did_document_storage, actor_b).await?;

    let mut conn = establish_connection().unwrap();
    let db_friend = crate::schema::appview::friend::table
        .filter(crate::schema::appview::friend::did.eq(&actor_a.did))
        .filter(crate::schema::appview::friend::targetdid.eq(&actor_b.did))
        .first::<Friend>(&mut conn);
    let mut is_friends = db_friend.is_ok();

    if !is_friends {
        let db_friend = crate::schema::appview::friend::table
            .filter(crate::schema::appview::friend::did.eq(&actor_b.did))
            .filter(crate::schema::appview::friend::targetdid.eq(&actor_a.did))
            .first::<Friend>(&mut conn);
        is_friends = db_friend.is_ok();
    }

    Ok(is_friends)
}

pub async fn get_actor_block_status(client: &Client, did_document_storage: &LruDidDocumentStorage, actor_a: &str, actor_b: &str) -> Result<(bool, bool)> {
    let actor_a = get_actor(client, did_document_storage, actor_a).await?;
    let actor_b = get_actor(client, did_document_storage, actor_b).await?;

    let mut conn = establish_connection().unwrap();
    let db_block = crate::schema::appview::block::table
        .filter(crate::schema::appview::block::did.eq(&actor_a.did))
        .filter(crate::schema::appview::block::targetdid.eq(&actor_b.did))
        .first::<Block>(&mut conn);
    let mut is_blocked = db_block.is_ok();

    if !is_blocked {
        let db_block = crate::schema::appview::block::table
            .filter(crate::schema::appview::block::did.eq(&actor_b.did))
            .filter(crate::schema::appview::block::targetdid.eq(&actor_a.did))
            .first::<Block>(&mut conn);
        is_blocked = db_block.is_ok();
    }

    let db_block = crate::schema::appview::block::table
        .filter(crate::schema::appview::block::did.eq(&actor_a.did))
        .filter(crate::schema::appview::block::targetdid.eq(&actor_b.did))
        .first::<Block>(&mut conn);
    let mut is_blocking = db_block.is_ok();

    if !is_blocking {
        let db_block = crate::schema::appview::block::table
            .filter(crate::schema::appview::block::did.eq(&actor_b.did))
            .filter(crate::schema::appview::block::targetdid.eq(&actor_a.did))
            .first::<Block>(&mut conn);
        is_blocking = db_block.is_ok();
    }

    Ok((is_blocking, is_blocked))
}