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
            PermissionFlag,
            check::{PermissionCheck, PermissionCheckTestable, scoped_permission},
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
            .has_permissions(PermissionFlag::VIEW_CONTENT | PermissionFlag::MANAGE_TENTS))?
    );

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
