use std::collections::HashSet;

use anyhow::Result;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use common::{fetch_record, fetch_record_list, GetRecordResponse};
use diesel::result::Error::NotFound;
use reqwest::Client;

use diesel::prelude::*;
use crate::database::actors::get_actor;
use crate::DNS_RESOLVER;
use campground_lexicon::gg::campground::profile::ProfilePost as ProfilePostRecord;
use crate::database::{
    establish_connection,
    models::appview::Actor,
    models::appview::Profile,
    models::appview::ProfilePost,
};
use crate::schema::appview::profile_post;

pub async fn get_profile_posts(client: &Client, did_document_storage: &LruDidDocumentStorage, uri: String) -> Result<Vec<ProfilePost>, &'static str> {
    let mut conn = establish_connection().unwrap();

    let post_uri = if uri.starts_with("at://") { uri } else { format!("at://{}", uri) };
    let split_uri: Vec<&str> = post_uri.split('/').collect();

    let author_did = split_uri[2];

    // TODO: Resolve handles to DIDs
    if !author_did.starts_with("did:") {
        return Err("Invalid author DID");
    }

    // at://did:.../gg.campground.profile.posts/...
    let fetch_format = match split_uri[..] {
        [_, _, did, "gg.campground.profile.post", _] | [_, _, did, "gg.campground.profile.post"]
            => format!("at://{}/gg.campground.profile.post/%", did),
        [_, _, _, _, _] | [_, _, _, _]
            => { return Err("Invalid record uri") }
        _
            => { return Err("Invalid URI format") }
    };

    // TODO: Firehose auto-update?
    let post_records = fetch_record_list::<ProfilePostRecord>(
        &author_did,
        "gg.campground.profile.post",
        client,
        did_document_storage,
        &DNS_RESOLVER
    )
        .await
        .map_err(|_| "Failed to fetch post records")
        .unwrap();


    let post_query = 
        if split_uri.len() > 4 {
            profile_post::table.filter(
                profile_post::parenturi
                    .eq(
                        post_uri.clone()
                    )
            )
            // .load(&mut conn)
            .load::<ProfilePost>(&mut conn)
        } else {
            profile_post::table.filter(
                profile_post::uri
                    .like(fetch_format)
                    .and(
                        profile_post::parenturi
                            .is_null()
                    )
            )
            .load::<ProfilePost>(&mut conn)
        };

    let parent_uri = if split_uri.len() > 4 { Some(post_uri.clone()) } else { None };
        
    let mut posts = post_query.expect("Error loading profile post");
    // Because Rust
    let post_uris: Vec<String> = posts.iter().map(|x| x.uri.clone()).clone().collect();

    let posts_to_add =
        post_records
            .records
            .iter()
            .filter(|x| x.value.parent_uri == parent_uri)
            .filter(|x| !post_uris.contains(&x.uri));

    for post in posts_to_add {
        let inserted = insert_post_record_into_db(post, author_did).await?;
        // To add to response
        posts.push(inserted);
    }
    
    Ok(posts)
}

pub async fn insert_post_record_into_db(post_record: &GetRecordResponse<ProfilePostRecord>, author_did: &str) -> Result<ProfilePost, &'static str> {
    let indexed_time = chrono::Utc::now().naive_utc().to_string();

    let profile_post = ProfilePost {
        cid: post_record.cid.to_string(),
        uri: post_record.uri.to_string(),
        parent_uri: post_record.value.parent_uri.clone(),
        content: match &post_record.value.content {
            None => "".to_string(),
            Some(x) => x.to_string()
        },
        indexed_at: indexed_time.clone(),
        author: author_did.to_string(),
        replies: match &post_record.value.replies {
            Some(vec) => vec.iter().map(|x| { Some(x.clone()) }).collect(),
            None => Vec::new(),
        },
        tags: match &post_record.value.tags {
            Some(vec) => vec.iter().map(|x| { Some(x.clone()) }).collect(),
            None => Vec::new(),
        },
        created_at: match post_record.value.created_at {
            Some(created_at) => created_at.to_string(),
            None => indexed_time.clone()
        },
        updated_at: match post_record.value.updated_at {
            Some(updated_at) => Some(updated_at.to_string()),
            None => None
        },
    };

    let mut conn = establish_connection().unwrap();
    diesel::insert_into(crate::schema::appview::profile_post::table)
        .values(&profile_post)
        .execute(&mut conn)
        .map_err(|e| { println!("Err: {}", e); "Error inserting into db" })?;

    Ok(profile_post)
}

pub fn get_authors_from_posts(posts: Vec<ProfilePost>) -> HashSet<String> {
    posts
        .iter()
        // at://did:.../
        .map(|x| { x.uri.split('/').collect::<Vec<&str>>()[2].to_string() })
        .collect::<HashSet<String>>()
}

pub fn populate_profile_posts_with_authors(posts: Vec<ProfilePost>, author_profiles: Vec<(Actor, Profile)>) -> Vec<(Actor, Profile, ProfilePost)> {
    return posts
        .iter()
        .clone()
        .map(|x| {
            let actor_tuple = author_profiles.iter().find(|y| y.0.did == x.author);
            (actor_tuple, x)
        })
        .filter(|x| x.0.is_some())
        .map(|x| {
            let a: &(Actor, Profile) = x.0.unwrap();
            (a.0.clone(), a.1.clone(), x.1.clone())
        })
        .collect::<Vec<(Actor, Profile, ProfilePost)>>();
}
pub async fn get_profile_post(client: &Client, did_document_storage: &LruDidDocumentStorage, uri: &str) -> Result<(Actor, ProfilePost), &'static str> {
    let mut conn = establish_connection().unwrap();
    
    let post_uri = if uri.starts_with("at://") { uri.to_string() } else { format!("at://{}", uri) };
    let split_uri: Vec<&str> = post_uri.split('/').collect();
    
    let author_did = split_uri[2].to_string();
    
    // TODO: Resolve handles to DIDs
    if !author_did.starts_with("did:") {
        return Err("Invalid author DID");
    }
    
    let author_actor = get_actor(client, did_document_storage, split_uri[2]).await.map_err(|_| "Error fetching actor")?;
    
    // at://did:.../gg.campground.profile.posts/...
    let fetch_format = match split_uri[..] {
        [_, _, did, "gg.campground.profile.post", post_tid]
        => format!("at://{did}/gg.campground.profile.post/{post_tid}", did = did, post_tid = post_tid),
        [_, _, _, _, _] | [_, _, _, _]
        => { return Err("Invalid uri") }
        _
        => { return Err("Invalid URI format") }
    };
    
    let post_tid = split_uri[4];

    let post_query = 
        profile_post::table.filter(
            profile_post::uri
                .eq(fetch_format)
        )
        .first::<ProfilePost>(&mut conn);

    match post_query {
        Ok(profile_post) => Ok((author_actor.clone(), profile_post)),
        Err(NotFound) => {
            let post_record = fetch_record::<ProfilePostRecord>(
                &author_did.clone(),
                "gg.campground.profile.post",
                post_tid,
                client,
                did_document_storage,
                &DNS_RESOLVER
            )
                .await
                .map_err(|_| { "Error fetching post record" })?;

            let profile_post = insert_post_record_into_db(&post_record, author_did.as_str()).await?;

            Ok((author_actor.clone(), profile_post))
        },
        Err(_) => Err("Error querying profile post")
    }
}