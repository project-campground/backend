use std::ops::{Add, Sub};

use appview_schema::{
    models::appview::{Tent, TentCategory},
    schema::appview::{self, tent_category},
};
use campground_lexicon::gg::campground::tent::TentCategoryView;
use chrono::Utc;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, dsl::not, sql_types::BigInt,
};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{
    api::gg::campground::tent::move_tent::check_bonfire_existence,
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_select_first_error,
        permissions::{
            ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner,
        },
        ws::event_next_category,
    },
    realtime::data::ReactiveSubject,
    views::tents::tent_category_view,
    xrpc::{
        campsite::CategoryInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MoveCategoryBody {
    bonfire_id: Option<String>,
    position: Option<i32>,
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.moveCategory?<category_id>", data = "<body>")]
pub async fn move_category(
    auth: CategoryInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    category_id: &str,
    body: Json<MoveCategoryBody>,
) -> Result<Json<TentCategoryView>> {
    let MoveCategoryBody {
        bonfire_id,
        position,
    } = &body.into_inner();
    if bonfire_id.is_none() && position.is_none() {
        return Err(XRPCError::BadRequest(
            "Expected at least one property in the body".to_string(),
        ));
    } else if bonfire_id
        .clone()
        .map_or(true, |bonfire_id| auth.category.bonfire_id == bonfire_id)
        || position.map_or(true, |position| auth.category.priority == position)
    {
        // Failing seems like a bad DX if there is some weird bug happening
        return Ok(Json(tent_category_view(&auth.category)));
    }

    expect_permission!(has_leveled_perms_or_owner(
        &auth.campsite,
        &auth.category.bonfire_id,
        Some(auth.category.id.clone()),
        None,
        &auth.member,
        GeneralPermissionConsts::MANAGE_TENTS,
        ContentPermissionConsts::VIEW_CONTENT
    ));

    let moved_bonfire = bonfire_id
        .clone()
        .unwrap_or(auth.category.bonfire_id.clone());

    // To make sure they are not moving to category that doesn't exist
    if moved_bonfire != auth.category.bonfire_id {
        check_bonfire_existence(&auth.campsite, &auth.member, &moved_bonfire).await?;
    }
    let current_date = Utc::now().naive_utc();

    let mut conn = establish_connection().unwrap();

    if let Some(position) = position {
        make_room_for_category(&auth.category.bonfire_id, *position)?;
    }

    let updated_category = diesel::update(crate::schema::appview::tent_category::table)
        .filter(appview::tent_category::id.eq(&auth.category.id))
        .set((
            // All the new settings
            appview::tent_category::priority.eq(position.unwrap_or(auth.category.priority)),
            appview::tent_category::bonfireid.eq(&moved_bonfire),
            // Mandatory
            appview::tent_category::updatedat.eq(current_date),
            appview::tent_category::updatedby.eq(&auth.actor.did),
        ))
        .load::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;

    // If category was moved to another bonfire, then move all the tents along with it
    if moved_bonfire != auth.category.bonfire_id {
        diesel::update(crate::schema::appview::tent::table)
            .filter(appview::tent::categoryid.eq(auth.category.id))
            .set((
                // All the new settings
                appview::tent::bonfireid.eq(moved_bonfire),
            ))
            .load::<Tent>(&mut conn)
            .map_err(handle_select_first_error)?;
    }

    let updated_category = updated_category.first().unwrap();

    event_next_category(
        event_subject,
        &updated_category,
        false,
        "CategoryMoved",
        tent_category_view(updated_category),
    );

    return Ok(Json(tent_category_view(updated_category)));
}

fn make_room_for_category(bonfire_id: &str, position: i32) -> Result<(), XRPCError> {
    let mut conn = establish_connection().unwrap();

    let exists_categories_there = tent_category::table
        .filter(
            tent_category::bonfireid
                .eq(bonfire_id)
                .and(tent_category::priority.eq(position)),
        )
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;

    // No other categories to update
    if exists_categories_there < 1 {
        return Ok(());
    }

    // Make other categories go above it (since client is expected to subtract 1 when putting above a category already)
    diesel::update(crate::schema::appview::tent_category::table)
        .filter(
            tent_category::bonfireid
                .eq(bonfire_id)
                // Don't move all tents; it would change nothing. Instead, make a gap for that position specifically
                .and(tent_category::priority.ge(position))
                .and(not(tent_category::priority
                    .cast::<BigInt>()
                    .add(1)
                    .gt(i32::MAX as i64))),
        )
        .set((tent_category::priority.eq(tent_category::priority.sub(1)),))
        .execute(&mut conn)
        .map_err(handle_select_first_error)?;

    Ok(())
}
