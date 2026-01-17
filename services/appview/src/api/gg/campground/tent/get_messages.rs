use appview_schema::{models::appview::{Actor, Profile, Tent, TentMessage}, schema::appview::{self, profile, tent_message}};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::{GetTentMessagesOutput, TentMessageViewBasic, TentMessageViewWithReplies};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::{actors::get_actor, establish_connection}, helpers::tents::{tent_message_view_basic, tent_message_view_with_replies}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.tent.getMessages?<tent_id>&<limit>&<offset>")]
pub async fn get_messages(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, tent_id: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Json<GetTentMessagesOutput>> {    
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    if limit < 1 || limit > 100 {
        return Err(XRPCError::BadRequest("Expected limit query to be between and including 1 and 100".to_string()));
    }
    
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?;

    let mut conn = establish_connection().unwrap();
    let actor = get_actor(client, did_document_storage, actor_did.as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;

    // Can be given invalid UUID; Be descriptive
    let tent_id_uuid = Uuid::try_parse(tent_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'tent_id' query to be a valid UUID".to_string()))
        ?;
    let tent = &crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::id
                .eq(tent_id_uuid)
        )
        .first::<Tent>(&mut conn)
        .map_err(|x|
            match x {
                diesel::result::Error::NotFound => XRPCError::NotFound,
                _ => XRPCError::InternalServerError,
            }
        )?;

    // Not in the campsite to view that
    if !actor.campsites.contains(&Some(tent.campsite_id.to_string())) {
        return Err(XRPCError::Forbidden("User cannot view campsite that they are not member of".to_string()));
    } else if tent.r#type != 0 {
        return Err(XRPCError::BadRequest("Cannot get messages of non-text tents".to_string()));
    }

    let message_queries: Vec<(TentMessage, Option<Actor>, Option<Profile>)> = tent_message::table
        .order_by(tent_message::createdat.desc())
        .limit(limit)
        .offset(offset)
        .left_join(
            appview::actor::table
                .on(
                    tent_message::createdby.eq(
                        appview::actor::did
                    )
                )
        )
        .left_join(
            profile::table
                .on(
                    tent_message::createdby.eq(
                        profile::creator
                    )
                )
        )
        .filter(
            crate::schema::appview::tent_message::tentid
                .eq(tent_id_uuid)
        )
        .load::<(TentMessage, Option<Actor>, Option<Profile>)>(&mut conn)
        .expect("Error loading tent messages");
    // Annoying. Hopefully better join can be done with author and replies
    let message_ids: Vec<Uuid> = message_queries.iter().map(|x| x.0.id).collect();
    let additional_reply_ids = message_queries
        .iter()
        .flat_map(|x|
            x
            .0
            .replying_to
            .iter()
            .filter_map(|y| y.clone())
        )
        // Already fetched
        .filter(|x|
            !message_ids.contains(x)
        )
        .collect::<Vec<Uuid>>();
    // To not re-fetch same messages
    let mut replies =
        if additional_reply_ids.len() < 1 {
            vec![]
        } else {
            tent_message::table
                .filter(
                    crate::schema::appview::tent_message::id
                        .eq_any(additional_reply_ids)
                )
                .left_join(
                    appview::actor::table
                        .on(
                            tent_message::createdby.eq(
                                appview::actor::did
                            )
                        )
                )
                .left_join(
                    profile::table
                        .on(
                            tent_message::createdby.eq(
                                profile::creator
                            )
                        )
                )
                .load::<(TentMessage, Option<Actor>, Option<Profile>)>(&mut conn)
                .expect("Error loading tent message replies")
        };
    replies.extend(message_queries.clone());

    let messages = message_queries
        .iter()
        .map(|x| {
            let replies_view =
                replies
                    .iter()
                    .filter(|y|
                        x.0.replying_to.contains(&Some(y.0.id))
                    )
                    .map(|y|
                        tent_message_view_basic(tent, &y.0, &y.1, &y.2)
                    )
                    .collect::<Vec<TentMessageViewBasic>>();
            tent_message_view_with_replies(tent, &x.0, replies_view, &x.1, &x.2)
        })
        .collect::<Vec<TentMessageViewWithReplies>>();

    return Ok(Json(GetTentMessagesOutput { messages }));
}
