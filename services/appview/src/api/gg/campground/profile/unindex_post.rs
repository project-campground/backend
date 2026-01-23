use appview_schema::{models::appview::ProfilePost, schema::appview::profile_post};
use campground_lexicon::gg::campground::profile::ProfilePostViewBasic;
use diesel::{ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use rocket::serde::json::Json;

use crate::{
    database::{establish_connection, profile_posts::{self, delete_post_record_from_db}, profiles}, helpers::{posts::profile_post_view_basic, views::profile_record}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[post("/xrpc/gg.campground.profile.unindexPost?<uri>")]
pub async fn unindex_post(auth: Authorization<'_>, uri: &str) -> Result<Json<ProfilePostViewBasic>> {
    let (resolved_uri, author_did, post_tid) = profile_posts::resolve_post_uri(uri)
        .map_err(|_| XRPCError::BadRequest("Incorrect 'uri' URI format".to_string()))?;

    if post_tid.is_none() {
        return Err(XRPCError::BadRequest("Expected post TID in the 'uri' query (at://.../.../post_tid_here".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let post_query = 
        profile_post::table
            .filter(
                profile_post::uri
                    .eq(resolved_uri)
            )
            .first::<ProfilePost>(&mut conn);
    let (actor, main_post) = profile_posts::get_single_profile_post(auth.client, auth.did_document_storage, post_query, author_did.as_str(), post_tid.unwrap().as_str(), |x| x).await.map_err(|_| XRPCError::NotFound)?;
    let (_, db_profile) = profiles::get_profile(auth.client, auth.did_document_storage, actor.did.as_str()).await.map_err(|_| XRPCError::NotFound)?;

    if auth.actor_did != actor.did {
        return Err(XRPCError::Forbidden("Only author of the post can unindex it".to_string()));
    }

    delete_post_record_from_db(main_post.uri.clone(), main_post.parent_uri.clone()).await.map_err(|_| XRPCError::InternalServerError)?;

    return Ok(Json(profile_post_view_basic(&actor, &profile_record(db_profile), &main_post.clone())));
}