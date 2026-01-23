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

#[get("/xrpc/gg.campground.profile.getReplies?<uri>&<limit>&<offset>")]
pub async fn get_replies(_auth: OptionalAuthorization<'_>, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Json<GetProfilePostRepliesOutput>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    if limit > 100 || limit < 1 || offset < 0 {
        return Err(XRPCError::BadRequest("Expected 'limit' query to be between (and including) 1 and 100, as well as 'offset' query to be positive integer or 0".to_string()));
    }

    let (resolved_uri, author_did, post_tid) = profile_posts::resolve_post_uri(uri)
        .map_err(|_| XRPCError::BadRequest("Incorrect 'uri' URI format".to_string()))?;

    if post_tid.is_none() {
        return Err(XRPCError::BadRequest("Expected post TID in the 'uri' query (at://.../.../post_tid_here".to_string()));
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