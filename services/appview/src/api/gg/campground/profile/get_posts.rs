use appview_schema::models::appview::ProfilePost;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::profile::GetProfilePostsOutput;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{
    database::{DbConnection, establish_connection, profile_posts, profiles}, helpers::{posts::{profile_post_view_basic, profile_post_view_parented}, views::profile_record}, schema::appview::profile_post, util::post_authors, xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.profile.getPosts?<actor>", rank = 3)]
pub async fn get_posts_defaults(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actor: &str) -> Result<Json<GetProfilePostsOutput>> {
    get_posts(auth, client, did_document_storage, actor, 50, 0, false).await
}

#[get("/xrpc/gg.campground.profile.getPosts?<actor>&<offset>", rank = 2)]
pub async fn get_posts_defaults_with_offset(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actor: &str, offset: i64) -> Result<Json<GetProfilePostsOutput>> {
    get_posts(auth, client, did_document_storage, actor, 50, offset, false).await
}

#[get("/xrpc/gg.campground.profile.getPosts?<actor>&<limit>&<offset>&<replies>")]
pub async fn get_posts(_auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actor: &str, limit: i64, offset: i64, replies: bool) -> Result<Json<GetProfilePostsOutput>> {
    if limit > 100 || limit < 1 || offset < 0 {
        return Err(XRPCError::BadRequest);
    }

    if !actor.starts_with("did:") {
        return Err(XRPCError::BadRequest);
    }

    let conn = establish_connection().unwrap();
    let uri_format = format!("at://{}/gg.campground.profile.post/%", actor);

    if replies {
        get_posts_with_replies(client, did_document_storage, uri_format, actor, limit, offset, replies, conn)
            .await
    } else {
        get_posts_no_replies(client, did_document_storage, uri_format, actor, limit, offset, replies, conn)
            .await
    }
}

async fn get_posts_no_replies(client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri_format: String, actor: &str, limit: i64, offset: i64, replies: bool, mut conn: DbConnection) -> Result<Json<GetProfilePostsOutput>> {
    let posts_query =
        profile_post::table.filter(
            profile_post::uri
            .like(uri_format)
                .and(
                    profile_post::parenturi
                        .is_null()
                )
            )
        .order_by(profile_post::indexedat.desc())
        .limit(limit)
        .offset(offset)
        .load::<ProfilePost>(&mut conn)
        .expect("Error querying posts");
    let posts =
        profile_posts::fill_profile_posts_with_records(client, did_document_storage, actor.to_string(), None, posts_query)
        .await
            .map_err(|_| XRPCError::InternalServerError)?;

    // let posts = profile_posts::get_profile_posts(client, did_document_storage, uri.to_string(), limit, offset).await.map_err(|_| XRPCError::NotFound)?;
    let actors = post_authors::get_authors_from_posts(posts.clone(), replies);
    let profiles = profiles::get_profiles(client, did_document_storage, actors.into_iter().collect()).await.map_err(|_| XRPCError::NotFound)?;

    let mapped_posts =
        post_authors::populate_profile_posts_with_authors(posts, profiles)
            .iter()
            .map(|x| profile_post_view_parented(&x.0, &profile_record(x.1.clone()), &x.2, &None))
            .collect();

    return Ok(Json(GetProfilePostsOutput { posts: mapped_posts }))

}

async fn get_posts_with_replies(client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, uri_format: String, actor: &str, limit: i64, offset: i64, replies: bool, mut conn: DbConnection) -> Result<Json<GetProfilePostsOutput>> {
    let (pp1, pp2) = diesel::alias!(profile_post as pp1, profile_post as pp2);

    // TODO: Left join, but while populating with authors from fetched profile posts?
    let posts_query = 
        pp1
            .order_by(pp1.field(profile_post::indexedat).desc())
            .limit(limit)
            .offset(offset)
            .left_join(
                pp2
                    .on(
                        pp1
                            .field(profile_post::parenturi)
                            .eq(
                                pp2
                                    .field(profile_post::uri)
                                    .nullable()
                            )
                    )
            )
            .filter(
                pp1
                    .field(profile_post::uri)
                    .like(uri_format)
            )
            // .select((profile_post::all_columns, profile_post::all_columns))
            .load::<(ProfilePost, Option<ProfilePost>)>(&mut conn)
            .expect("Error querying posts");
    // To use existing methods
    let posts_query_no_parent: Vec<ProfilePost> =
        posts_query
            .iter()
            .map(|x| x.0.clone())
            .collect();
    let posts =
        profile_posts::fill_profile_posts_with_records(client, did_document_storage, actor.to_string(), None, posts_query_no_parent)
            .await
            .map_err(|_| XRPCError::InternalServerError)?;

    let actors = post_authors::get_authors_from_posts(posts.clone(), replies);
    let profiles = profiles::get_profiles(client, did_document_storage, actors.into_iter().collect()).await.map_err(|_| XRPCError::NotFound)?;

    // Now introduce parents
    let populated_parents = post_authors::populate_profile_posts_with_authors_from_iter(
        posts_query
            .into_iter()
            .filter(|x| x.1.is_some())
            .map(|x| x.1.clone().unwrap()),
        profiles.clone()
    );
    let mut parents = populated_parents
        .iter()
        .map(|x| profile_post_view_basic(&x.0, &profile_record(x.1.clone()), &x.2));

    // now with parents as well
    let mapped_posts =
        post_authors::populate_profile_posts_with_authors(posts, profiles.clone())
            .iter()
            .map(|x| {
                // If there is no parent, no point
                let found_parent =
                    if x.2.parent_uri.is_none() { &None }
                    else {
                        let parent_uri = x.2.parent_uri.clone().unwrap();
                        &parents.find(|y| { y.uri == parent_uri })
                    };

                profile_post_view_parented(
                    &x.0,
                    &profile_record(x.1.clone()),
                    &x.2,
                    found_parent
                )
            })
            .collect();

    return Ok(Json(GetProfilePostsOutput { posts: mapped_posts }))

}
