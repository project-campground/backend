use appview_schema::models::appview::ProfilePost;
use appview_schema::schema::appview::profile_post;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::profile::ProfilePostViewDetailed;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    TextExpressionMethods,
};
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::database::establish_connection;
use crate::util::post_authors;
use crate::views::posts::{profile_post_view_basic, profile_post_view_detailed};
use crate::views::profiles::profile_record;
use crate::{
    database::profile_posts,
    database::profiles,
    xrpc::{
        auth::OptionalAuthorization,
        error::{Result, XRPCError},
    },
};

#[get("/xrpc/gg.campground.profile.getPost?<uri>&<limit>&<offset>")]
pub async fn get_post(
    _auth: OptionalAuthorization<'_>,
    client: &State<Client>,
    did_document_storage: &State<LruDidDocumentStorage>,
    uri: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Json<ProfilePostViewDetailed>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    if limit > 100 || limit < 1 || offset < 0 {
        return Err(XRPCError::BadRequest("Expected 'limit' query to be between (and including) 1 and 100, as well as 'offset' query to be positive integer or 0".to_string()));
    }

    let (resolved_uri, author_did, post_tid) = profile_posts::resolve_post_uri(uri)
        .map_err(|_| XRPCError::BadRequest("Incorrect 'uri' URI format".to_string()))?;

    if post_tid.is_none() {
        return Err(XRPCError::BadRequest(
            "Expected post TID in the 'uri' query (at://.../.../post_tid_here".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();

    let (pp1, pp2) = diesel::alias!(profile_post as pp1, profile_post as pp2);
    let (author_actor, (main_post, parent_post)) = profile_posts::get_single_profile_post(
        client,
        did_document_storage,
        pp1.order_by(pp1.field(profile_post::indexedat).desc())
            .limit(limit)
            .offset(offset)
            .left_join(
                pp2.on(pp1
                    .field(profile_post::parenturi)
                    .eq(pp2.field(profile_post::uri).nullable())),
            )
            .filter(pp1.field(profile_post::uri).like(&resolved_uri))
            // .select(((ProfilePost, Option<ProfilePost>), Vec<ProfilePost>)::as_select())
            .first::<(ProfilePost, Option<ProfilePost>)>(&mut conn),
        &author_did,
        &post_tid.unwrap(),
        |x| (x, None),
    )
    .await
    .map_err(|_| XRPCError::NotFound)?;

    // TODO: Left join, but while populating with authors from fetched profile posts?
    let mut replies = profile_post::table
        .filter(profile_post::parenturi.eq(resolved_uri.clone()))
        // .load(&mut conn)
        .order_by(profile_post::indexedat.desc())
        .limit(limit)
        .offset(offset)
        .load::<ProfilePost>(&mut conn)
        .expect("Error loading profile post");
    profile_posts::fill_profile_posts_with_records(
        client,
        did_document_storage,
        &author_did,
        Some(resolved_uri.clone()),
        false,
        &mut replies,
    )
    .await?;

    let mut actors = post_authors::get_authors_from_posts(&replies, true);
    actors.insert(author_actor.did);

    // If is a bit more janky
    match parent_post.clone() {
        Some(x) => actors.insert(x.author),
        None => false,
    };

    let profiles = profiles::get_profiles(
        client,
        did_document_storage,
        actors.clone().into_iter().collect(),
    )
    .await
    .map_err(|_| XRPCError::NotFound)?;

    let mapped_replies =
        post_authors::populate_profile_posts_with_authors(replies.clone(), &profiles)
            .iter()
            .map(|x| profile_post_view_basic(&x.0, &profile_record(x.1.clone()), &x.2))
            .collect();

    // If it can't be found anyway,
    let parent_found_author = parent_post.clone().and_then(|x| {
        let parent_author = x.author;
        profiles.iter().find(|y| y.0.did == parent_author)
    });

    let found_author = profiles
        .iter()
        .find(|x| x.0.did == actors.iter().last().unwrap().clone());
    let author = match found_author {
        Some(x) => x,
        None => return Err(XRPCError::InternalServerError),
    };

    let author_record = profile_record(author.1.clone());

    let parent = if parent_found_author.is_some() {
        let parent_author_unwrapped = parent_found_author.unwrap();
        parent_post.clone().map(|x| {
            profile_post_view_basic(
                &parent_author_unwrapped.0,
                &profile_record(parent_author_unwrapped.1.clone()),
                &x,
            )
        })
    } else {
        None
    };

    return Ok(Json(profile_post_view_detailed(
        &author.0,
        &author_record,
        &main_post,
        mapped_replies,
        parent.as_ref(),
    )));
}
