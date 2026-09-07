use appview_schema::schema::appview::{campsite_permission, tent, tent_category};
use campground_lexicon::gg::campground::tent::TentCategoryView;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::establish_connection,
    expect_permission,
    helpers::{
        api::handle_all_db_errors,
        permissions::{
            ContentPermissionConsts, GeneralPermissionConsts, has_leveled_perms_or_owner,
        },
        ws::event_next_category,
    },
    realtime::data::ReactiveSubject,
    views::tents::tent_category_view,
    xrpc::{campsite::CategoryInfo, error::Result},
};

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.tent.deleteCategory?<category_id>")]
pub async fn delete_category(
    auth: CategoryInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    category_id: &str,
) -> Result<Json<TentCategoryView>> {
    expect_permission!(has_leveled_perms_or_owner(
        &auth.campsite,
        &auth.category.bonfire_id,
        Some(auth.category.id.clone()),
        None,
        &auth.member,
        GeneralPermissionConsts::MANAGE_TENTS,
        ContentPermissionConsts::VIEW_CONTENT
    ));

    let mut conn = establish_connection().unwrap();

    conn.transaction(|conn| {
        diesel::delete(tent_category::table)
            .filter(tent_category::id.eq(auth.category.id))
            .execute(conn)?;
        diesel::delete(campsite_permission::table)
            .filter(campsite_permission::categoryid.eq(auth.category.id))
            .execute(conn)?;
        diesel::update(tent::table)
            .filter(tent::categoryid.eq(auth.category.id))
            .set(tent::categoryid.eq::<Option<Uuid>>(None))
            .execute(conn)
    })
    .map_err(handle_all_db_errors)?;

    let view = tent_category_view(&auth.category);
    event_next_category(
        event_subject,
        &auth.category,
        true,
        "CategoryDeleted",
        &view,
    );

    return Ok(Json(view));
}
