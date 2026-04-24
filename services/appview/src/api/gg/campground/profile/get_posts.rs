use appview_schema::models::appview::ProfilePost;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::profile::{GetProfilePostsOutput, ProfilePostViewBasic};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
    RunQueryDsl, TextExpressionMethods,
};
use itertools::Itertools;
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::{
    database::{DbConnection, establish_connection, profile_posts, profiles},
    helpers::api::handle_all_db_errors,
    schema::appview::profile_post,
    util::post_authors,
    views::{
        posts::{profile_post_view_basic, profile_post_view_parented},
        profiles::profile_record,
    },
    xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/gg.campground.profile.getPosts?<actor>&<limit>&<offset>&<replies>")]
pub async fn get_posts(
    _auth: OptionalAuthorization<'_>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    actor: &str,
    limit: Option<i64>,
    offset: Option<i64>,
    replies: Option<bool>,
) -> Result<Json<GetProfilePostsOutput>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let replies = replies.unwrap_or(false);

    if limit > 100 || limit < 1 || offset < 0 {
        return Err(XRPCError::BadRequest("Expected 'limit' query to be between (and including) 1 and 100, as well as 'offset' query to be positive integer or 0".to_string()));
    }

    if !actor.starts_with("did:") {
        return Err(XRPCError::BadRequest(
            "Incorrect 'actor' query Atprotocol DID format".to_string(),
        ));
    }

    let conn = establish_connection().unwrap();
    let uri_format = format!("at://{}/gg.campground.profile.post/%", actor);

    if replies {
        get_posts_with_replies(
            client,
            did_document_storage,
            uri_format,
            actor,
            limit,
            offset,
            replies,
            conn,
        )
        .await
    } else {
        get_posts_no_replies(
            client,
            did_document_storage,
            uri_format,
            actor,
            limit,
            offset,
            replies,
            conn,
        )
        .await
    }
}

async fn get_posts_no_replies(
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    uri_format: String,
    actor: &str,
    limit: i64,
    offset: i64,
    replies: bool,
    mut conn: DbConnection,
) -> Result<Json<GetProfilePostsOutput>> {
    let mut posts = profile_post::table
        .filter(
            profile_post::uri
                .like(uri_format)
                .and(profile_post::parenturi.is_null()),
        )
        .order_by(profile_post::indexedat.desc())
        .limit(limit)
        .offset(offset)
        .load::<ProfilePost>(&mut conn)
        .map_err(handle_all_db_errors)?;
    profile_posts::fill_profile_posts_with_records(
        client,
        did_document_storage,
        &actor,
        None,
        false,
        &mut posts,
    )
    .await?;

    // let posts = profile_posts::get_profile_posts(client, did_document_storage, uri.to_string(), limit, offset).await.map_err(|_| XRPCError::NotFound)?;
    let actors = post_authors::get_authors_from_posts(&posts, replies);
    let profiles =
        profiles::get_profiles(client, did_document_storage, actors.into_iter().collect())
            .await
            .map_err(|_| XRPCError::NotFound)?;

    let mapped_posts = post_authors::populate_profile_posts_with_authors(posts, &profiles)
        .iter()
        .map(|x| profile_post_view_parented(&x.0, &profile_record(x.1.clone()), &x.2, &None))
        .collect();

    return Ok(Json(GetProfilePostsOutput {
        posts: mapped_posts,
    }));
}

async fn get_posts_with_replies(
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    uri_format: String,
    actor: &str,
    limit: i64,
    offset: i64,
    replies: bool,
    mut conn: DbConnection,
) -> Result<Json<GetProfilePostsOutput>> {
    let (pp1, pp2) = diesel::alias!(profile_post as pp1, profile_post as pp2);

    // TODO: Left join, but while populating with authors from fetched profile posts?
    let posts_query: &Vec<(ProfilePost, Option<ProfilePost>)> = &pp1
        .filter(pp1.field(profile_post::uri).like(uri_format))
        .order_by(pp1.field(profile_post::indexedat).desc())
        .left_join(
            pp2.on(pp1.field(profile_post::parenturi).is_not_null().and(
                pp1.field(profile_post::parenturi)
                    .assume_not_null()
                    .eq(pp2.field(profile_post::uri)),
            )),
        )
        .limit(limit)
        .offset(offset)
        // .select((profile_post::all_columns, profile_post::all_columns))
        .load::<(ProfilePost, Option<ProfilePost>)>(&mut conn)
        .map_err(handle_all_db_errors)?;

    println!(
        "--- Posts query: {:?}",
        posts_query
            .iter()
            .map(|(x, y)| format!(
                "{}: {:?} ({:?}) with parent {:?}",
                x.uri,
                x.content,
                x.parent_uri,
                y.clone().map(|x| x.content),
            ))
            .collect::<Vec<String>>()
    );

    // To use existing methods
    let mut posts: Vec<ProfilePost> = posts_query.iter().map(|x| x.0.clone()).collect();

    profile_posts::fill_profile_posts_with_records(
        client,
        did_document_storage,
        &actor,
        None,
        true,
        &mut posts,
    )
    .await?;

    let actors = post_authors::get_authors_from_posts(&posts, replies);
    let profiles =
        profiles::get_profiles(client, did_document_storage, actors.into_iter().collect())
            .await
            .map_err(|_| XRPCError::NotFound)?;

    // Now introduce parents
    let populated_parents = post_authors::populate_profile_posts_with_authors_from_iter(
        posts_query
            .into_iter()
            .filter(|x| x.1.is_some())
            .map(|x| x.1.clone().unwrap())
            .unique_by(|x| x.uri.clone()),
        &profiles,
    );
    let parents = populated_parents
        .iter()
        .map(|x| profile_post_view_basic(&x.0, &profile_record(x.1.clone()), &x.2))
        .collect::<Vec<ProfilePostViewBasic>>();

    // now with parents as well
    let mapped_posts = post_authors::populate_profile_posts_with_authors(posts, &profiles)
        .iter()
        .map(|x| {
            // If there is no parent, no point
            let found_parent = if x.2.parent_uri.is_none() {
                &None
            } else {
                let parent_uri = x.2.parent_uri.clone().unwrap();
                &parents.iter().find(|y| y.uri == parent_uri).cloned()
            };

            profile_post_view_parented(&x.0, &profile_record(x.1.clone()), &x.2, found_parent)
        })
        .collect();

    return Ok(Json(GetProfilePostsOutput {
        posts: mapped_posts,
    }));
}
