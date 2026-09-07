use std::ops::Add;

use appview_schema::{models::appview::Bonfire, schema::appview};
use campground_lexicon::gg::campground::bonfire::BonfireViewBasic;
use chrono::Utc;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, dsl::not, sql_types::BigInt,
};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{
            ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner,
        },
        ws::event_next_bonfire,
    },
    realtime::data::ReactiveSubject,
    views::bonfires::bonfire_view_basic,
    xrpc::{
        campsite::BonfireInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MoveBonfireBody {
    position: i32,
}

#[post(
    "/xrpc/gg.campground.bonfire.moveBonfire?<bonfire_id>",
    data = "<body>"
)]
pub async fn move_bonfire(
    auth: BonfireInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    bonfire_id: &str,
    body: Json<MoveBonfireBody>,
) -> Result<Json<BonfireViewBasic>> {
    let MoveBonfireBody { position } = &body.into_inner();

    if *position == auth.bonfire.priority {
        return Ok(Json(bonfire_view_basic(&auth.bonfire)));
    }

    let mut conn = establish_connection().unwrap();

    expect_permission!(has_leveled_perms_or_owner(
        &auth.campsite,
        &bonfire_id,
        None,
        None,
        &auth.member,
        GeneralPermissionConsts::MANAGE_BONFIRES,
        ContentPermissionConsts::VIEW_CONTENT
    ));

    make_room_for_bonfire(&auth.bonfire.campsite_id, *position)?;

    let current_date = Utc::now().naive_utc();

    let bonfire = diesel::update(appview::bonfire::table)
        .filter(
            appview::bonfire::campsiteid
                .eq(&auth.bonfire.campsite_id)
                .and(appview::bonfire::id.eq(bonfire_id)),
        )
        .set((
            appview::bonfire::priority.eq(position),
            appview::bonfire::updatedby.eq(&auth.actor.did),
            appview::bonfire::updatedat.eq(current_date),
        ))
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    let bonfire = bonfire.first().unwrap();

    let view = bonfire_view_basic(bonfire);
    event_next_bonfire(event_subject, &bonfire, false, "BonfireMoved", &view);

    return Ok(Json(view));
}

fn make_room_for_bonfire(campsite_id: &str, position: i32) -> Result<(), XRPCError> {
    let mut conn = establish_connection().unwrap();

    let bonfire_exists_there = appview::bonfire::table
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(appview::bonfire::priority.eq(position)),
        )
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;

    if bonfire_exists_there < 1 {
        return Ok(());
    }

    // Make other bonfires go below it (since client is expected to add 1 when putting below a bonfire already)
    diesel::update(crate::schema::appview::bonfire::table)
        .filter(
            appview::bonfire::campsiteid
                .eq(campsite_id)
                // Update only bonfires at that position and below
                .and(appview::bonfire::priority.ge(position))
                // Make sure it can even go down
                .and(not(appview::bonfire::priority
                    .cast::<BigInt>()
                    .add(1)
                    .gt(i32::MAX as i64))),
        )
        .set((appview::bonfire::priority.eq(appview::bonfire::priority + 1),))
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(())
}
