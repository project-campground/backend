use std::ops::Add;

use appview_schema::{models::appview::{Bonfire, Campsite, CampsiteMember, Tent, TentCategory}, schema::appview};
use campground_lexicon::gg::campground::tent::TentViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, dsl::not, sql_types::BigInt};
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::establish_connection, expect_permission, helpers::{api::handle_select_first_error, permissions::{ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner}, tents::tent_view_basic, ws::event_next_tent}, realtime::data::ReactiveSubject, xrpc::{
    campsite::TentInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MoveTentBody {
    bonfire_id: Option<String>,
    category_id: Option<String>,
    position: Option<i32>,
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.moveTent?<tent_id>", data = "<body>")]
pub async fn move_tent(auth: TentInfo<'_>, event_subject: &State<ReactiveSubject>, tent_id: &str, body: Json<MoveTentBody>) -> Result<Json<TentViewBasic>> {    
    let inner_body = &body.into_inner();

    // No reason to do anything with the request
    if inner_body.bonfire_id.is_none() && inner_body.category_id.is_none() && inner_body.position.is_none() {
        return Err(XRPCError::BadRequest("Expected at least one property in the body".to_string()));
    }

    expect_permission!(
        has_leveled_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, GeneralPermissionConsts::MANAGE_TENTS, ContentPermissionConsts::VIEW_CONTENT)
    );

    let remove_category = inner_body.category_id.clone().map_or(false, |x| x == "");
    let category_id =
        if inner_body.category_id.is_none() || remove_category {
            None
        } else {
            Some(Uuid::try_parse(inner_body.category_id.clone().unwrap().as_str())
                .map_err(|_| XRPCError::BadRequest("Invalid category_id UUID format".to_string()))?)
        };

    let mut conn = establish_connection().unwrap();

    let moved_bonfire = inner_body.bonfire_id.clone().unwrap_or(auth.tent.bonfire_id.clone());

    // To make sure they are not moving to category that doesn't exist
    if !remove_category && category_id.map_or(false, |x| Some(x) != auth.tent.category_id) {
        check_category_existence(&auth.campsite, &auth.member, &moved_bonfire, category_id.unwrap()).await?;
    } else if moved_bonfire != auth.tent.bonfire_id {
        check_bonfire_existence(&auth.campsite, &auth.member, &moved_bonfire).await?
    }
    let moved_category = if remove_category { None } else { category_id.or(auth.tent.category_id.clone()) };

    // Make room for the tent; if there is already a tent in that position, make sure position is slightly more unique and is more consistent
    // among the client and so would the experience (since if it also gets sorted by ID, it would be confusing why sometimes tent refuses to move)
    if let Some(position) = inner_body.position {
        make_room_for_tent(&auth.tent.bonfire_id, moved_category, position)?;
    }

    let current_date = Utc::now().naive_utc();
    
    let updated_tent = diesel::update(crate::schema::appview::tent::table)
        .filter(
            appview::tent::id
                .eq(
                    auth.tent.id.clone()
                )
        )
        .set((
            // All the new settings
            appview::tent::priority
                .eq(inner_body.position.clone().unwrap_or(auth.tent.priority)),
            appview::tent::categoryid
                .eq(moved_category),
            appview::tent::bonfireid
                .eq(&moved_bonfire),
            // Mandatory
            appview::tent::updatedat
                .eq(current_date),
            appview::tent::updatedby
                .eq(&auth.actor.did),
        ))
        .load::<Tent>(&mut conn)
        .map_err(handle_select_first_error)?;

    let updated_tent = updated_tent.first().unwrap();

    event_next_tent(event_subject, &auth.tent, false, "TentMoved", tent_view_basic(updated_tent));

    return Ok(Json(tent_view_basic(updated_tent)));
}

fn make_room_for_tent(bonfire_id: &str, category_id: Option<Uuid>, position: i32) -> Result<(), XRPCError> {
    let mut conn = establish_connection().unwrap();

    println!("Category ID: {:?}, Position: {:?}", category_id, position);
    let exists_tents_there = appview::tent::table
        .filter(
            appview::tent::bonfireid
                .eq(bonfire_id)
                // Only see if there are tents in the same position AND category
                // It doesn't affect how it appears if it's in different category
                .and(
                    appview::tent::categoryid
                        .eq(category_id)
                )
                .and(
                    appview::tent::priority
                        .eq(position)
                )
        )
        .count()
        .first::<i64>(&mut conn)
        .map_err(handle_select_first_error)?;
    println!("Tents exist there: {:?}", exists_tents_there);

    // No other tents to update
    if exists_tents_there < 1 {
        return Ok(());
    }

    // Make other tents go below it (since client is expected to add 1 when putting below a tent already)
    let updated = diesel::update(crate::schema::appview::tent::table)
        .filter(
            appview::tent::bonfireid
                .eq(bonfire_id)
                .and(
                    // Don't really care about tents outside the category, their position doesn't affect anything
                    appview::tent::categoryid
                        .eq(category_id)
                )
                // Update only tents at that position and below
                .and(
                    appview::tent::priority
                        .ge(position)
                )
                // Make sure it can even go down
                .and(
                    not(
                        appview::tent::priority
                            .cast::<BigInt>()
                            .add(1)
                            .gt(i32::MAX as i64)
                    )
                )
        )
        .set((
            appview::tent::priority
                .eq(
                    appview::tent::priority
                        .add(1)
                ),
        ))
        .load::<Tent>(&mut conn)
        .map_err(handle_select_first_error)?;

    println!("Updated tents: {:?}", updated);

    Ok(())
}

async fn check_category_existence(campsite: &Campsite, member: &CampsiteMember, moved_bonfire_id: &str, category_id: Uuid) -> Result<String, XRPCError> {    
    let mut conn = establish_connection().unwrap();

    let tent_categories = &crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::id
                .eq(category_id)
        )
        .load::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;

    if tent_categories.len() == 0 {
        return Err(XRPCError::BadRequest("Category supplied in 'category_id' does not exist".to_string()));
    }
    
    let tent_category = tent_categories.first().unwrap();

    if tent_category.campsite_id != campsite.id {
        return Err(XRPCError::BadRequest("Category supplied in 'category_id' does not belong to the same campsite as tent".to_string()));
    } else if tent_category.bonfire_id != moved_bonfire_id {
        return Err(XRPCError::BadRequest("Category supplied in 'category_id' does not belong to the same bonfire as supplied in 'bonfire_id' or already existing bonfire".to_string()));
    }

    if !has_leveled_perms_or_owner(campsite, moved_bonfire_id, Some(category_id), None, member, GeneralPermissionConsts::MANAGE_TENTS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    return Ok(tent_category.bonfire_id.clone());
}

pub async fn check_bonfire_existence(campsite: &Campsite, member: &CampsiteMember, moved_bonfire_id: &str) -> Result<(), XRPCError> {    
    let mut conn = establish_connection().unwrap();

    let bonfires = &crate::schema::appview::bonfire::table
        .filter(
            crate::schema::appview::bonfire::id
                .eq(moved_bonfire_id)
                .and(
                    crate::schema::appview::bonfire::campsiteid
                        .eq(
                            &campsite.id
                        )
                )
        )
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    if bonfires.len() == 0 {
        return Err(XRPCError::BadRequest("Bonfire supplied in 'bonfire_id' does not exist".to_string()));
    }

    if !has_leveled_perms_or_owner(campsite, moved_bonfire_id, None, None, member, GeneralPermissionConsts::MANAGE_TENTS, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    return Ok(());
}