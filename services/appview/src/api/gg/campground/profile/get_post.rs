use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::profile::ProfilePostViewDetailed;
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::helpers::posts::{profile_post_view_basic, profile_post_view_detailed};
use crate::helpers::views::profile_record;
use crate::{
    database::profiles,
    database::profile_posts,
    xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.profile.getPost?<uri>")]
pub async fn get_post(_auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str) -> Result<Json<ProfilePostViewDetailed>> {
    let main_post = profile_posts::get_profile_post(client, did_document_storage, uri).await.map_err(|e| XRPCError::NotFound)?;

    let replies = profile_posts::get_profile_posts(client, did_document_storage, uri.to_string()).await.map_err(|_| XRPCError::NotFound)?;
    
    let mut actors = profile_posts::get_authors_from_posts(replies.clone());
    actors.insert(main_post.0.did);
    
    let profiles = profiles::get_profiles(client, did_document_storage, actors.clone().into_iter().collect()).await.map_err(|_| XRPCError::NotFound)?;

    let mapped_replies =
        profile_posts::populate_profile_posts_with_authors(replies.clone(), profiles.clone())
            .iter()
            .map(|x| profile_post_view_basic(&x.0, &profile_record(x.1.clone()), &x.2))
            .collect();

    let found_author = profiles.iter().find(|x| { x.0.did == actors.iter().last().unwrap().clone() });

    let author = match found_author {
        Some(x) => x,
        None => { return Err(XRPCError::InternalServerError) }
    };

    let author_record = profile_record(author.1.clone());

    return Ok(Json(profile_post_view_detailed(&author.0, &author_record, &main_post.1, mapped_replies)));
}