use std::ops::Add;

use campground_lexicon::chat::campground::bonfire::BonfireViewBasic;
use cds_schema::{models::cds::Bonfire, schema::cds};
use chrono::Utc;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl,
    dsl::not, sql_types::BigInt,
};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{
    database::{DbConnection, establish_connection},
    expect_permission,
    helpers::{
        api::handle_all_db_errors,
        permissions::{
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
        },
        ws::event_next_bonfire,
    },
    realtime::data::ReactiveSubject,
    views::bonfires::bonfire_view_basic,
    xrpc::{campsite::BonfireInfo, error::Result},
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MoveBonfireBody {
    position: i32,
}

#[post(
    "/xrpc/chat.campground.bonfire.moveBonfire?<bonfire_id>",
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

    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| scoped_permission(
                permission.fetch_roles(),
                &auth.campsite.id,
                bonfire_id,
                None,
                None
            )
            .has_permissions(PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_BONFIRES))?
    );

    let mut conn = establish_connection().unwrap();

    let bonfires = conn
        .transaction(|conn| {
            make_room_for_bonfire(conn, &auth.bonfire.campsite, *position)?;

            let current_date = Utc::now().naive_utc();

            let bonfires = diesel::update(cds::bonfire::table)
                .filter(
                    cds::bonfire::campsite
                        .eq(&auth.bonfire.campsite)
                        .and(cds::bonfire::id.eq(bonfire_id)),
                )
                .set((
                    cds::bonfire::priority.eq(position),
                    cds::bonfire::updated_by.eq(&auth.actor.did),
                    cds::bonfire::updated_at.eq(current_date),
                ))
                .load::<Bonfire>(conn)?;

            QueryResult::Ok(bonfires)
        })
        .map_err(handle_all_db_errors)?;

    let bonfire = bonfires.first().unwrap();
    let view = bonfire_view_basic(bonfire);

    event_next_bonfire(event_subject, bonfire, false, "BonfireMoved", &view);

    return Ok(Json(view));
}

fn make_room_for_bonfire(
    conn: &mut DbConnection,
    campsite_id: &str,
    position: i32,
) -> Result<(), diesel::result::Error> {
    let bonfire_exists_there = cds::bonfire::table
        .filter(
            cds::bonfire::campsite
                .eq(campsite_id)
                .and(cds::bonfire::priority.eq(position)),
        )
        .count()
        .first::<i64>(conn)?;

    if bonfire_exists_there < 1 {
        return Ok(());
    }

    // Make other bonfires go below it (since client is expected to add 1 when putting below a bonfire already)
    diesel::update(crate::schema::cds::bonfire::table)
        .filter(
            cds::bonfire::campsite
                .eq(campsite_id)
                // Update only bonfires at that position and below
                .and(cds::bonfire::priority.ge(position))
                // Make sure it can even go down
                .and(not(cds::bonfire::priority
                    .cast::<BigInt>()
                    .add(1)
                    .gt(i32::MAX as i64))),
        )
        .set((cds::bonfire::priority.eq(cds::bonfire::priority + 1),))
        .load::<Bonfire>(conn)?;

    Ok(())
}
