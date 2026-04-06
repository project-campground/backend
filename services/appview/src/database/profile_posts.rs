use anyhow::Result;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use chrono::NaiveDateTime;
use common::{fetch_record, fetch_record_list, GetRecordResponse};
use diesel::dsl::sql;
use diesel::result::Error::NotFound;
use diesel::sql_types::{Array, VarChar};
use reqwest::Client;

use diesel::prelude::*;
use crate::database::actors::get_actor;
use crate::DNS_RESOLVER;
use campground_lexicon::gg::campground::profile::ProfilePost as ProfilePostRecord;
use crate::database::{
    establish_connection,
    models::appview::Actor,
    models::appview::ProfilePost,
};
use crate::schema::appview::profile_post;

pub fn resolve_post_uri(uri: &str) -> Result<(String, String, Option<String>), &'static str> {
    let post_uri = if uri.starts_with("at://") { uri.to_string() } else { format!("at://{}", uri) };
    let split_uri: Vec<&str> = post_uri.split('/').collect();
    
    let author_did = split_uri[2].to_string();
    
    // TODO: Resolve handles to DIDs
    if !author_did.starts_with("did:") {
        return Err("Invalid author DID");
    }

    // at://did:.../gg.campground.profile.posts/...
    let uri_formatted = match split_uri[..] {
        [_, _, did, "gg.campground.profile.post", post_tid]
            => format!("at://{did}/gg.campground.profile.post/{post_tid}", did = did, post_tid = post_tid),
        [_, _, did, "gg.campground.profile.post"]
            => format!("at://{did}/gg.campground.profile.post/%", did = did),
        _ => { return Err("Invalid URI format") }
    };

    Ok((uri_formatted, author_did, if split_uri.len() < 5 { None } else { Some(split_uri[4].to_string()) }))
}

pub async fn fill_profile_posts_with_records(client: &Client, did_document_storage: &LruDidDocumentStorage, author_did: &str, parent_uri: Option<String>, optional_parent: bool, posts: &mut Vec<ProfilePost>) -> Result<(), &'static str> {
    // TODO: Firehose auto-update?
    let post_records = fetch_record_list::<ProfilePostRecord>(
        author_did,
        "gg.campground.profile.post",
        client,
        did_document_storage,
        &DNS_RESOLVER
    )
        .await
        .map_err(|err| format!("Failed to fetch post records: {:?}", err))
        .unwrap();

    // Because Rust
    let post_uris: Vec<String> = posts.iter().map(|x| x.uri.clone()).collect();

    // Some of the new record that weren't found before
    let posts_to_add: &Vec<ProfilePost> =
        &post_records
            .records
            .iter()
            .filter(|x|
                (optional_parent || x.value.parent_uri == parent_uri)
                && !post_uris.contains(&x.uri)
            )
            .map(|x| db_post_from_only_record(x))
            .collect();

    let posts_modified: &Vec<ProfilePost> =
        &post_records
            .records
            .iter()
            .filter(|x|
                posts
                    .iter()
                    .find(|y| y.uri == x.uri)
                    .map_or(
                        false,
                        // TODO Likely to be modified when embeds exist
                        |y| x.value.content.is_some() && y.content != x.value.content.clone().unwrap()
                    )
            )
            .map(|x| db_post_from_only_record(x))
            .collect();
    
    insert_post_list_into_db(posts_to_add.clone()).await?;
    update_post_list_in_db(posts_modified.clone()).await?;

    posts.extend(posts_to_add.clone());
    
    let fmt = "%Y-%m-%d %H:%M:%S.%f";

    posts
        .sort_by(|a, b|
            NaiveDateTime::parse_from_str(b.indexed_at.as_str(), fmt).unwrap().cmp(&NaiveDateTime::parse_from_str(a.indexed_at.as_str(), fmt).unwrap())
        );

    Ok(())
}
pub async fn get_single_profile_post<T>(client: &Client, did_document_storage: &LruDidDocumentStorage, post_query: Result<T, diesel::result::Error>, author_did: &str, post_tid: &str, _fn: fn(ProfilePost) -> T) -> Result<(Actor, T), &'static str> {
    let author_actor = get_actor(client, did_document_storage, author_did).await.map_err(|_| "Error fetching actor")?;

    match post_query {
        Ok(profile_post) => Ok((author_actor.clone(), profile_post)),
        Err(NotFound) => {
            let post_record = fetch_record::<ProfilePostRecord>(
                author_did,
                "gg.campground.profile.post",
                post_tid,
                client,
                did_document_storage,
                &DNS_RESOLVER
            )
                .await
                .map_err(|_| { "Error fetching post record" })?;

            let profile_post = insert_post_into_db(&post_record, author_did).await?;
            
            Ok((author_actor.clone(), _fn(profile_post)))
        },
        Err(_) => Err("Error querying profile post")
    }
}

pub async fn delete_post_record_from_db(uri: &str, parent_uri: Option<String>) -> Result<usize, &'static str> {
    let mut conn = establish_connection().unwrap();

    // To have an up-to-date reply list
    if parent_uri.is_some() {
        diesel::update(profile_post::table)
                .filter(
                    profile_post::uri
                        .eq(parent_uri.unwrap())
                )
                .set(
                    profile_post::replies
                        .eq(
                            sql("array_remove(replies, ")
                                .bind::<VarChar, _>(uri)
                                .sql(")")
                        )
                )
                .execute(&mut conn)
                .expect("Could not update parent post reply list");
    }

    // Finally delete the post from DB
    let deleted_count =
        diesel::delete(
            profile_post::table
                .filter(
                    profile_post::uri
                        .eq(uri)
                )
        )
        .execute(&mut conn)
        .map_err(|_| "Error deleting post from db")?;

    Ok(deleted_count)
}

fn db_post_from_only_record(post_record: &GetRecordResponse<ProfilePostRecord>) -> ProfilePost {
    let x_author_did = post_record.uri.split("/").skip(2).next().map_or("did:plc:null", |x| x); 
    db_post_from_record(post_record, x_author_did)
}

fn db_post_from_record(post_record: &GetRecordResponse<ProfilePostRecord>, author_did: &str) -> ProfilePost {
    let indexed_time = chrono::Utc::now().naive_utc().to_string();

    ProfilePost {
        cid: post_record.cid.to_string(),
        uri: post_record.uri.to_string(),
        parent_uri: post_record.value.parent_uri.clone(),
        content: match &post_record.value.content {
            None => "".to_string(),
            Some(x) => x.to_string()
        },
        indexed_at: indexed_time.clone(),
        author: author_did.to_string(),
        replies: Vec::new(),
        created_at: match post_record.value.created_at {
            Some(created_at) => created_at.to_string(),
            None => indexed_time.clone()
        },
        updated_at: match post_record.value.updated_at {
            Some(updated_at) => Some(updated_at.to_string()),
            None => None
        },
    }
}

async fn insert_post_list_into_db(posts: Vec<ProfilePost>) -> Result<(), &'static str> {
    let mut conn = establish_connection().unwrap();

    diesel::insert_into(crate::schema::appview::profile_post::table)
        .values(posts.clone())
        .execute(&mut conn)
        .map_err(|e| { println!("Err: {}", e); "Error inserting into db" })?;

    
    // Stringed tuple inserted posts and their parents to keep a list of parents to update
    let parents: Vec<String> =
        posts
            .clone()
            .iter()
            // Don't care about ones without parents
            .filter(|x| x.parent_uri.is_some())
            // Stringed tuple
            .map(|x| {
                format!("{};{}", x.uri.clone(), x.parent_uri.clone().unwrap())
            })
            .collect();

    if parents.len() < 1 {
        return Ok(());
    }

    // Basically a complicated way of fetching specific list of replies for each parent instead of individually updating and spamming queries
    let reply_update_sql = sql("(SELECT array_agg(split_part(unnest, ';', 1)) FROM ( SELECT unnest(")
        .bind::<Array<VarChar>, _>(parents.clone())
        .sql(") ) WHERE split_part(unnest, ';', 2) = \"appview\".\"profile_post\".\"uri\")");

    diesel::update(profile_post::table)
        .filter(
            profile_post::uri.eq_any(parents.iter().map(|x| x.split(";").collect::<Vec<&str>>()[1]))
        )
        .set(profile_post::replies
            .eq(profile_post::replies.concat(reply_update_sql))
        )
        .execute(&mut conn)
        .expect("Could not update parent post reply lists");

    Ok(())
}

async fn insert_post_into_db(post_record: &GetRecordResponse<ProfilePostRecord>, author_did: &str) -> Result<ProfilePost, &'static str> {
    let mut conn = establish_connection().unwrap();
    let profile_post = db_post_from_record(post_record, author_did);
    
    diesel::insert_into(crate::schema::appview::profile_post::table)
        .values(&profile_post)
        .execute(&mut conn)
        .map_err(|e| { println!("Err: {}", e); "Error inserting into db" })?;

    if post_record.value.parent_uri.is_none() {
        return Ok(profile_post);
    }

    
    let parent_uri = post_record.value.parent_uri.clone().unwrap();

    diesel::update(profile_post::table)
        .filter(
            profile_post::uri.eq(parent_uri.clone())
        )
        .set(profile_post::replies
            .eq(profile_post::replies.concat(vec![ post_record.uri.clone() ]))
        )
        .execute(&mut conn)
        .expect("Could not update parent post reply lists");

    Ok(profile_post)
}

async fn update_post_list_in_db(posts: Vec<ProfilePost>) -> Result<(), &'static str> {
    // Don't need to do anything
    if posts.len() < 1 {
        return Ok(());
    }

    let mut conn = establish_connection().unwrap();
    
    // Stringed tuple inserted posts and their parents to keep a list of parents to update
    let update_tuple_string: Vec<String> =
        posts
            .clone()
            .iter()
            // Stringed tuple
            .map(|x| {
                format!("{};{}", x.uri.clone(), x.content.clone())
            })
            .collect();

    // Basically a complicated way of fetching specific list of replies for each parent instead of individually updating and spamming queries
    let content_update_sql = sql("(SELECT ((array_agg(array_to_string((string_to_array(unnest, ';'))[2:], ';')))[1]) FROM ( SELECT unnest(")
        .bind::<Array<VarChar>, _>(update_tuple_string.clone())
        .sql(") ) WHERE split_part(unnest, ';', 1) = \"appview\".\"profile_post\".\"uri\")");

    diesel::update(profile_post::table)
        .filter(
            profile_post::uri.eq_any(update_tuple_string.iter().map(|x| x.split(";").collect::<Vec<&str>>()[0]))
        )
        .set(profile_post::content
            .eq(content_update_sql)
        )
        .execute(&mut conn)
        .expect("Could not update post list");

    Ok(())
}
