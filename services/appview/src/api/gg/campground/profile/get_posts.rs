use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::profile::GetProfilePostsOutput;
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{
    database::{profile_posts, profiles}, helpers::{posts::profile_post_view_basic, views::profile_record}, xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.profile.getPosts?<uri>")]
pub async fn get_posts(_auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str) -> Result<Json<GetProfilePostsOutput>> {
    let posts =  profile_posts::get_profile_posts(client, did_document_storage, uri.to_string()).await.map_err(|e| { println!("E0: {}", e); XRPCError::NotFound })?;
    let actors = profile_posts::get_authors_from_posts(posts.clone());
    let profiles = profiles::get_profiles(client, did_document_storage, actors.into_iter().collect()).await.map_err(|e| { println!("E1: {}", e); XRPCError::NotFound })?;

    let mapped_posts =
        profile_posts::populate_profile_posts_with_authors(posts, profiles)
            .iter()
            .map(|x| profile_post_view_basic(&x.0, &profile_record(x.1.clone()), &x.2))
            .collect();
    
    return Ok(Json(GetProfilePostsOutput { posts: mapped_posts }))
}