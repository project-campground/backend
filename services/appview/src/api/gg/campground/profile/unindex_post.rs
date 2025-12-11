use appview_schema::{models::appview::ProfilePost, schema::appview::profile_post};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::profile::ProfilePostViewBasic;
use diesel::{ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{
    database::{establish_connection, profile_posts::{self, delete_post_record_from_db}, profiles}, helpers::{posts::profile_post_view_basic, views::profile_record}, xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.profile.unindexPost?<uri>")]
pub async fn unindex_post(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri: &str) -> Result<Json<ProfilePostViewBasic>> {
    let (resolved_uri, author_did, post_tid) = profile_posts::resolve_post_uri(uri)
        .map_err(|_| XRPCError::BadRequest)?;

    if post_tid.is_none() {
        return Err(XRPCError::BadRequest);
    }

    let mut conn = establish_connection().unwrap();

    let post_query = 
        profile_post::table
            .filter(
                profile_post::uri
                    .eq(resolved_uri)
            )
            .first::<ProfilePost>(&mut conn);
    let (actor, main_post) = profile_posts::get_single_profile_post(client, did_document_storage, post_query, author_did.as_str(), post_tid.unwrap().as_str(), |x| x).await.map_err(|_| XRPCError::NotFound)?;
    let (_, db_profile) = profiles::get_profile(client, did_document_storage, actor.did.as_str()).await.map_err(|_| XRPCError::NotFound)?;

    if auth.1.jose.issuer.is_none() {
        return Err(XRPCError::Forbidden);
    }

    delete_post_record_from_db(main_post.uri.clone(), main_post.parent_uri.clone()).await.map_err(|_| XRPCError::InternalServerError)?;

    return Ok(Json(profile_post_view_basic(&actor, &profile_record(db_profile), &main_post.clone())));
}