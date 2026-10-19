use appview_schema::{models::appview::Tent, schema::appview};
use campground_lexicon::gg::campground::tent::TentViewBasic;
use chrono::Utc;
use diesel::QueryDsl;
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, RunQueryDsl, sql_types::Bool};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::gg::campground::tent::util::{check_bonfire_existence, check_category_existence},
    database::{DbConnection, establish_connection},
    expect_permission,
    helpers::{
        api::handle_all_db_errors,
        permissions::{
            ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner,
        },
        ws::event_next_tent,
    },
    make_room_for_items,
    realtime::data::ReactiveSubject,
    views::tents::tent_view_basic,
    xrpc::{
        campsite::TentInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[serde(untagged)]
pub enum MoveTentBody {
    #[serde(rename_all = "camelCase")]
    ToBonfire {
        bonfire_id: String,
        position: i32,
    },
    #[serde(rename_all = "camelCase")]
    ToCategory {
        category_id: String,
        position: i32,
    },
    ToPosition {
        position: i32,
    },
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.moveTent?<tent_id>", data = "<body>")]
pub async fn move_tent(
    auth: TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    tent_id: &str,
    body: Json<MoveTentBody>,
) -> Result<Json<TentViewBasic>> {
    expect_permission!(has_leveled_perms_or_owner(
        &auth.campsite,
        &auth.tent.bonfire_id,
        auth.tent.category_id.clone(),
        Some(auth.tent.id),
        &auth.member,
        GeneralPermissionConsts::MANAGE_TENTS,
        ContentPermissionConsts::VIEW_CONTENT
    ));

    match body.into_inner() {
        MoveTentBody::ToBonfire {
            bonfire_id,
            position,
        } => move_tent_to_bonfire(auth, event_subject, bonfire_id, position).await,
        MoveTentBody::ToCategory {
            category_id,
            position,
        } => move_tent_to_category(auth, event_subject, category_id, position).await,
        MoveTentBody::ToPosition { position } => {
            move_tent_to_pos(auth, event_subject, position).await
        }
    }
}

fn handle_moved_tents(
    auth: &TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    result: (Vec<Tent>, Vec<(Uuid, i32)>),
) -> Result<Json<TentViewBasic>, XRPCError> {
    let tent = result.0.first().unwrap();

    let view = tent_view_basic(tent);
    event_next_tent(event_subject, &auth.tent, false, "TentMoved", &view);

    Ok(Json(view))
}

async fn move_tent_to_pos(
    auth: TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    position: i32,
) -> Result<Json<TentViewBasic>, XRPCError> {
    let mut conn = establish_connection().unwrap();

    handle_moved_tents(
        &auth,
        event_subject,
        set_tent_pos_unchecked(
            &mut conn,
            &auth,
            &auth.tent.bonfire_id,
            auth.tent.category_id,
            position,
        )?,
    )
}

async fn move_tent_to_bonfire(
    auth: TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    bonfire_id: String,
    position: i32,
) -> Result<Json<TentViewBasic>, XRPCError> {
    // Maybe a client had failed checked
    if bonfire_id == auth.tent.bonfire_id {
        return Ok(Json(tent_view_basic(&auth.tent)));
    }

    let mut conn = establish_connection().unwrap();

    check_bonfire_existence(&mut conn, &auth.campsite, &auth.member, &bonfire_id)?;

    handle_moved_tents(
        &auth,
        event_subject,
        set_tent_pos_unchecked(&mut conn, &auth, &bonfire_id, None, position)?,
    )
}

async fn move_tent_to_category(
    auth: TentInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    category_id: String,
    position: i32,
) -> Result<Json<TentViewBasic>, XRPCError> {
    let category_id = if category_id == "" {
        None
    } else {
        Some(uuid::Uuid::parse_str(&category_id).map_err(|_| {
            XRPCError::BadRequest("Invalid UUID format for category_id".to_string())
        })?)
    };

    // Maybe a client had failed checked
    if category_id == auth.tent.category_id {
        return Ok(Json(tent_view_basic(&auth.tent)));
    }

    let mut conn = establish_connection().unwrap();

    let result = (if let Some(category_id) = category_id {
        move_tent_to_specified_category(&mut conn, &auth, category_id, position)
    } else {
        set_tent_pos_unchecked(&mut conn, &auth, &auth.tent.bonfire_id, None, position)
    })?;

    handle_moved_tents(&auth, event_subject, result)
}

fn move_tent_to_specified_category(
    conn: &mut DbConnection,
    auth: &TentInfo<'_>,
    category_id: Uuid,
    position: i32,
) -> Result<(Vec<Tent>, Vec<(Uuid, i32)>), XRPCError> {
    let new_bonfire = check_category_existence(conn, &auth.campsite, &auth.member, category_id)?;

    set_tent_pos_unchecked(conn, &auth, &new_bonfire, Some(category_id), position)
}

fn set_tent_pos_unchecked(
    conn: &mut DbConnection,
    auth: &TentInfo<'_>,
    bonfire_id: &str,
    category_id: Option<Uuid>,
    position: i32,
) -> Result<(Vec<Tent>, Vec<(Uuid, i32)>), XRPCError> {
    conn.transaction(|conn| {
        let also_moved = make_room_for_tent(conn, &auth.tent.bonfire_id, category_id, position)?;

        let current_date = Utc::now().naive_utc();

        Ok((
            diesel::update(crate::schema::appview::tent::table)
                .filter(appview::tent::id.eq(auth.tent.id))
                .set((
                    // All the new settings
                    appview::tent::priority.eq(position),
                    appview::tent::categoryid.eq(category_id),
                    appview::tent::bonfireid.eq(bonfire_id),
                    // Mandatory
                    appview::tent::updatedat.eq(current_date),
                    appview::tent::updatedby.eq(&auth.actor.did),
                ))
                .load::<Tent>(conn)?,
            also_moved,
        ))
    })
    .map_err(handle_all_db_errors)
}

fn make_room_for_tent(
    conn: &mut DbConnection,
    bonfire_id: &str,
    category_id: Option<Uuid>,
    position: i32,
) -> Result<Vec<(Uuid, i32)>, diesel::result::Error> {
    use diesel::sql_types::BigInt;
    use std::ops::Add;

    make_room_for_items!(
        crate::schema::appview::tent::table,
        crate::schema::appview::tent::id,
        crate::schema::appview::tent::priority,
        appview::tent::bonfireid
            .eq(bonfire_id)
            // Only see if there are tents in the same position AND category
            // It doesn't affect how it appears if it's in different category
            .and(
                appview::tent::categoryid
                    .is_null()
                    .and::<bool, Bool>(category_id == None)
                    .or(appview::tent::categoryid.eq(category_id)),
            );
        conn,
        position
    );
}
