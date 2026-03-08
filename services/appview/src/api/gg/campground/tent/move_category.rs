use appview_schema::{models::appview::{Tent, TentCategory}, schema::appview};
use campground_lexicon::gg::campground::tent::TentCategoryView;
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{api::gg::campground::tent::move_tent::check_bonfire_existence, database::establish_connection, helpers::{api::handle_select_first_error, permissions::{CampsitePermissionConsts, TentPermissionConsts, has_tent_perms_or_owner}, tents::tent_category_view, ws::event_next_category}, realtime::data::ReactiveSubject, xrpc::{
    campsite::CategoryInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MoveCategoryBody {
    bonfire_id: Option<String>,
    priority: Option<i32>,
}

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.moveCategory?<category_id>", data = "<body>")]
pub async fn move_category(auth: CategoryInfo<'_>, event_subject: &State<ReactiveSubject>, category_id: &str, body: Json<MoveCategoryBody>) -> Result<Json<TentCategoryView>> {    
    let inner_body = &body.into_inner();
    if inner_body.bonfire_id.is_none() && inner_body.priority.is_none() {
        return Err(XRPCError::BadRequest("Expected at least one property in the body".to_string()));
    }

    if !has_tent_perms_or_owner(&auth.campsite, &auth.category.bonfire_id, Some(auth.category.id.clone()), None, &auth.member, CampsitePermissionConsts::MANAGE_TENTS, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let moved_bonfire = inner_body.bonfire_id.clone().unwrap_or(auth.category.bonfire_id.clone());

    // To make sure they are not moving to category that doesn't exist
    if moved_bonfire != auth.category.bonfire_id {
        check_bonfire_existence(&auth.campsite, &auth.member, &moved_bonfire)
            .await?;
    }
    let current_date = Utc::now().naive_utc();
    
    let updated_category = diesel::update(crate::schema::appview::tent_category::table)
        .filter(
            appview::tent_category::id
                .eq(
                    &auth.category.id
                )
        )
        .set((
            // All the new settings
            appview::tent_category::priority
                .eq(inner_body.priority.clone().unwrap_or(auth.category.priority)),
            appview::tent_category::bonfireid
                .eq(&moved_bonfire),
            // Mandatory
            appview::tent_category::updatedat
                .eq(current_date),
            appview::tent_category::updatedby
                .eq(&auth.actor.did),
        ))
        .load::<TentCategory>(&mut conn)
        .map_err(handle_select_first_error)?;
    
    // If category was moved to another bonfire, then move all the tents along with it
    if moved_bonfire != auth.category.bonfire_id {
        diesel::update(crate::schema::appview::tent::table)
            .filter(
                appview::tent::categoryid
                    .eq(
                        auth.category.id
                    )
            )
            .set((
                // All the new settings
                appview::tent::bonfireid
                    .eq(moved_bonfire),
            ))
            .load::<Tent>(&mut conn)
            .map_err(handle_select_first_error)?;
    }

    let updated_category = updated_category.first().unwrap();

    event_next_category(event_subject, &auth.campsite.id, &auth.category.bonfire_id, auth.category.id, false, "CategoryMoved", tent_category_view(updated_category));

    return Ok(Json(tent_category_view(updated_category)));
}
