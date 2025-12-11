use appview_schema::models::appview::ProfilePost;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::profile::{GetProfilePostRepliesOutput, ProfilePostViewBasic};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;
use crate::database::{establish_connection, profile_posts};
use crate::schema::appview::profile_post;

use crate::{
    database::profiles, helpers::{posts::profile_post_view_basic, views::profile_record}, util::post_authors, xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.profile.getReplies?<uri>&<offset>", rank = 2)]
pub async fn get_replies_default_limit(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str, offset: i64) -> Result<Json<GetProfilePostRepliesOutput>> {
    get_replies(auth, client, did_document_storage, uri, 50, offset).await
}

#[get("/xrpc/gg.campground.profile.getReplies?<uri>&<limit>", rank = 3)]
pub async fn get_replies_default_offset(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str, limit: i64) -> Result<Json<GetProfilePostRepliesOutput>> {
    get_replies(auth, client, did_document_storage, uri, limit, 0).await
}

#[get("/xrpc/gg.campground.profile.getReplies?<uri>", rank = 4)]
pub async fn get_replies_default(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str) -> Result<Json<GetProfilePostRepliesOutput>> {
    get_replies(auth, client, did_document_storage, uri, 50, 0).await
}

#[get("/xrpc/gg.campground.profile.getReplies?<uri>&<limit>&<offset>")]
pub async fn get_replies(_auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str, limit: i64, offset: i64) -> Result<Json<GetProfilePostRepliesOutput>> {
    if limit > 100 || limit < 1 || offset < 0 {
        return Err(XRPCError::BadRequest);
    }

    let (resolved_uri, author_did, post_tid) = profile_posts::resolve_post_uri(uri)
        .map_err(|_| XRPCError::BadRequest)?;

    if post_tid.is_none() {
        return Err(XRPCError::BadRequest);
    }

    let mut conn = establish_connection().unwrap();

    let posts_query =
        profile_post::table.filter(
            profile_post::parenturi
                .eq(
                    resolved_uri.clone()
                )
        )
        // .load(&mut conn)
        .order_by(profile_post::indexedat.desc())
        .limit(limit)
        .offset(offset)
        .load::<ProfilePost>(&mut conn)
        .expect("Error loading profile post");
    let posts =
        profile_posts::fill_profile_posts_with_records(client, did_document_storage, author_did, Some(resolved_uri), posts_query)
            .await
            .map_err(|_| XRPCError::InternalServerError)?;

    // let posts = profile_posts::get_profile_posts(client, did_document_storage, uri.to_string(), limit, offset).await.map_err(|_| XRPCError::NotFound)?;
    let actors = post_authors::get_authors_from_posts(posts.clone(), false);
    let profiles = profiles::get_profiles(client, did_document_storage, actors.into_iter().collect()).await.map_err(|_| XRPCError::NotFound)?;

    let mapped_posts: Vec<ProfilePostViewBasic> =
        post_authors::populate_profile_posts_with_authors(posts, profiles)
            .iter()
            .map(|x| profile_post_view_basic(&x.0, &profile_record(x.1.clone()), &x.2))
            .collect();
    
    return Ok(Json(GetProfilePostRepliesOutput { posts: mapped_posts }))
}