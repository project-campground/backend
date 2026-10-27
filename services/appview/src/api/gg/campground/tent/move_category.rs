use appview_schema::{models::appview::TentCategory, schema::appview};
use campground_lexicon::gg::campground::tent::TentCategoryView;
use chrono::Utc;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::gg::campground::tent::util::check_bonfire_existence,
    database::{DbConnection, establish_connection},
    expect_permission,
    helpers::{
        api::handle_all_db_errors,
        permissions::{
            PermissionConsts,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
        },
        ws::event_next_category,
    },
    make_room_for_items,
    realtime::data::ReactiveSubject,
    views::tents::tent_category_view,
    xrpc::{
        campsite::CategoryInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
#[serde(untagged)]
pub enum MoveTentBody {
    ToBonfire { bonfire_id: String, position: i32 },
    ToPosition { position: i32 },
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.moveCategory?<category_id>", data = "<body>")]
pub async fn move_category(
    auth: CategoryInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    category_id: &str,
    body: Json<MoveTentBody>,
) -> Result<Json<TentCategoryView>> {
    expect_permission!(
        PermissionCheck::member(&auth.member)
            .is_owner(&auth.campsite)
            .skip_if_owner(|permission| scoped_permission(
                permission.fetch_roles(),
                &auth.campsite.id,
                &auth.category.bonfire_id,
                Some(&auth.category.id),
                None
            )
            .has_permissions(PermissionConsts::VIEW_CONTENT | PermissionConsts::MANAGE_TENTS))?
    );

    match body.into_inner() {
        MoveTentBody::ToBonfire {
            bonfire_id,
            position,
        } => move_tent_to_bonfire(auth, event_subject, bonfire_id, position).await,
        MoveTentBody::ToPosition { position } => {
            move_tent_to_pos(auth, event_subject, position).await
        }
    }
}

fn handle_moved_categories(
    auth: &CategoryInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    result: (Vec<TentCategory>, Vec<(Uuid, i32)>),
) -> Result<Json<TentCategoryView>, XRPCError> {
    let category = result.0.first().unwrap();

    let view = tent_category_view(category);
    event_next_category(event_subject, &auth.category, false, "CategoryMoved", &view);

    Ok(Json(view))
}

async fn move_tent_to_pos(
    auth: CategoryInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    position: i32,
) -> Result<Json<TentCategoryView>, XRPCError> {
    // Maybe a client had failed checked
    if auth.category.priority == position {
        return Ok(Json(tent_category_view(&auth.category)));
    }

    let mut conn = establish_connection().unwrap();

    let result =
        set_category_position_unchecked(&mut conn, &auth, &auth.category.bonfire_id, position)?;

    handle_moved_categories(&auth, event_subject, result)
}

async fn move_tent_to_bonfire(
    auth: CategoryInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    bonfire_id: String,
    position: i32,
) -> Result<Json<TentCategoryView>, XRPCError> {
    // Maybe a client had failed checked
    if bonfire_id == auth.category.bonfire_id {
        return Ok(Json(tent_category_view(&auth.category)));
    }

    let mut conn = establish_connection().unwrap();

    check_bonfire_existence(&mut conn, &auth.campsite, &auth.member, &bonfire_id)?;

    let result = set_category_position_unchecked(&mut conn, &auth, &bonfire_id, position)?;

    handle_moved_categories(&auth, event_subject, result)
}

fn set_category_position_unchecked(
    conn: &mut DbConnection,
    auth: &CategoryInfo<'_>,
    bonfire_id: &str,
    position: i32,
) -> Result<(Vec<TentCategory>, Vec<(Uuid, i32)>), XRPCError> {
    conn.transaction(|conn| {
        let moved_categories = make_room_for_category(conn, bonfire_id, position)?;

        let current_date = Utc::now().naive_utc();

        Ok((
            diesel::update(crate::schema::appview::tent_category::table)
                .filter(appview::tent_category::id.eq(auth.category.id.clone()))
                .set((
                    // All the new settings
                    appview::tent_category::priority.eq(position),
                    appview::tent_category::bonfireid.eq(bonfire_id),
                    // Mandatory
                    appview::tent_category::updatedat.eq(current_date),
                    appview::tent_category::updatedby.eq(&auth.actor.did),
                ))
                .load::<TentCategory>(conn)?,
            moved_categories,
        ))
    })
    .map_err(handle_all_db_errors)
}

fn make_room_for_category(
    conn: &mut DbConnection,
    bonfire_id: &str,
    position: i32,
) -> Result<Vec<(Uuid, i32)>, diesel::result::Error> {
    use diesel::BoolExpressionMethods;
    use diesel::QueryDsl;
    use diesel::sql_types::BigInt;
    use std::ops::Add;

    make_room_for_items!(
        crate::schema::appview::tent_category::table,
        crate::schema::appview::tent_category::id,
        crate::schema::appview::tent_category::priority,
        appview::tent_category::bonfireid
            .eq(bonfire_id);
        conn,
        position
    );
}
