use appview_schema::schema::appview::{campsite_permission, tent, tent_category};
use campground_lexicon::gg::campground::tent::TentCategoryView;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::establish_connection, helpers::{permissions::{GeneralPermissionConsts, ContentPermissionConsts, has_leveled_perms_or_owner}, tents::tent_category_view, ws::event_next_category}, realtime::data::ReactiveSubject, xrpc::{
        campsite::CategoryInfo, error::{Result, XRPCError}
    }
};

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.deleteCategory?<category_id>")]
pub async fn delete_category(auth: CategoryInfo<'_>, event_subject: &State<ReactiveSubject>, category_id: &str) -> Result<Json<TentCategoryView>> {
    if !has_leveled_perms_or_owner(&auth.campsite, &auth.category.bonfire_id, Some(auth.category.id.clone()), None, &auth.member, GeneralPermissionConsts::MANAGE_TENTS, ContentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    diesel::delete(tent_category::table)
        .filter(
            tent_category::id
                .eq(auth.category.id)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;
    diesel::delete(campsite_permission::table)
        .filter(
            campsite_permission::categoryid
                .eq(auth.category.id)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;

    // To make it easier to delete sections of tents
    diesel::update(tent::table)
        .filter(
            tent::categoryid
                .eq(auth.category.id)
        )
        .set(
            tent::categoryid
                .eq::<Option<Uuid>>(None)
        )
        .execute(&mut conn)
        .map_err(|_| XRPCError::InternalServerError)?;

    event_next_category(event_subject, &auth.category, true, "CategoryDeleted", tent_category_view(&auth.category));

    return Ok(Json(tent_category_view(&auth.category)));
}
