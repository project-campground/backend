use appview_schema::{
    models::appview::{CampsitePermission, Tent, TentCategory},
    schema::appview::campsite_permission,
};
use campground_lexicon::gg::campground::{
    permission::PermissionViewBasic,
    tent::{GetTentsOutput, TentCategoryView, TentViewBasic},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    helpers::api::handle_all_db_errors,
    views::{
        permissions::permission_view_basic,
        tents::{tent_category_view, tent_view_basic},
    },
    xrpc::error::Result,
};

pub async fn get_tents(
    actor: &str,
    campsite_id: &str,
    bonfire_id: &str,
) -> Result<Json<GetTentsOutput>> {
    let mut conn = establish_connection().unwrap();

    let categories = crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::tent_category::bonfireid.eq(bonfire_id)),
        )
        .load::<TentCategory>(&mut conn)
        .expect("Error loading tent categories")
        .iter()
        .map(tent_category_view)
        .collect::<Vec<TentCategoryView>>();
    let tents = crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::tent::bonfireid.eq(bonfire_id)),
        )
        .load::<Tent>(&mut conn)
        .expect("Error loading tents")
        .iter()
        .map(tent_view_basic)
        .collect::<Vec<TentViewBasic>>();
    let permissions = campsite_permission::table
        .filter(
            campsite_permission::bonfireid.eq(bonfire_id).and(
                campsite_permission::roleid
                    .is_not_null()
                    .or(campsite_permission::userid.eq(&actor)),
            ),
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?
        .iter()
        .map(permission_view_basic)
        .collect::<Vec<PermissionViewBasic>>();

    return Ok(Json(GetTentsOutput {
        tents,
        categories,
        permissions,
    }));
}
