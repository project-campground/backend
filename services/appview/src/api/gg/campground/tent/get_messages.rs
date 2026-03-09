#![
    allow(unused_variables)
]
use appview_schema::{models::appview::{Actor, CampsiteMember, Profile, TentMessage}, schema::appview::{self, campsite_member, profile, tent_message}};
use campground_lexicon::gg::campground::tent::{GetTentMessagesOutput, TentMessageViewBasic, TentMessageViewWithReplies};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::{
    database::establish_connection, helpers::{api::handle_all_db_errors, permissions::{TentPermissionConsts, has_tent_perms_or_owner}, tents::{tent_message_view_basic, tent_message_view_with_replies}}, xrpc::{
        campsite::TentInfo, error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.tent.getMessages?<tent_id>&<limit>&<offset>")]
pub async fn get_messages(auth: TentInfo<'_>, tent_id: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Json<GetTentMessagesOutput>> {    
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    if limit < 1 || limit > 100 {
        return Err(XRPCError::BadRequest("Expected limit query to be between and including 1 and 100".to_string()));
    }
    else if !has_tent_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, 0, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    // Not in the campsite to view that
    if auth.tent.r#type != 0 {
        return Err(XRPCError::BadRequest("Cannot get messages of non-text tents".to_string()));
    }

    let message_queries: Vec<(TentMessage, Option<Actor>, Option<Profile>, Option<CampsiteMember>)> = tent_message::table
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
        .left_join(
            campsite_member::table
                .on(
                    tent_message::createdby.eq(
                        campsite_member::userid
                    )
                        .and(
                            tent_message::campsiteid.eq(
                                campsite_member::campsiteid
                            )
                        )
                )
        )
        .filter(
            crate::schema::appview::tent_message::tentid
                .eq(auth.tent.id)
        )
        .load::<(TentMessage, Option<Actor>, Option<Profile>, Option<CampsiteMember>)>(&mut conn)
        .map_err(handle_all_db_errors)?;
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
                .left_join(
                    campsite_member::table
                        .on(
                            tent_message::createdby.eq(
                                campsite_member::userid
                            )
                                .and(
                                    tent_message::campsiteid.eq(
                                        campsite_member::campsiteid
                                    )
                                )
                        )
                )
                .load::<(TentMessage, Option<Actor>, Option<Profile>, Option<CampsiteMember>)>(&mut conn)
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
                        tent_message_view_basic(&auth.tent, &y.0, &y.1, &y.2, &y.3)
                    )
                    .collect::<Vec<TentMessageViewBasic>>();
            tent_message_view_with_replies(&auth.tent, &x.0, replies_view, &x.1, &x.2, &x.3)
        })
        .collect::<Vec<TentMessageViewWithReplies>>();

    return Ok(Json(GetTentMessagesOutput { messages }));
}
