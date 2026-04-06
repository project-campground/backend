use appview_schema::models::appview::CampsiteRole;
use campground_lexicon::gg::campground::role::{GetRolesOutput, RoleViewBasic};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    helpers::api::handle_select_first_error,
    views::roles::role_view_basic,
    xrpc::{campsite::CampsiteInfoBasic, error::Result},
};

#[get("/xrpc/gg.campground.role.getRoles?<campsite_id>")]
pub async fn get_roles(
    _auth: CampsiteInfoBasic<'_>,
    campsite_id: &str,
) -> Result<Json<GetRolesOutput>> {
    let mut conn = establish_connection().unwrap();

    let roles = crate::schema::appview::campsite_role::table
        .filter(crate::schema::appview::campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(role_view_basic)
        .collect::<Vec<RoleViewBasic>>();

    return Ok(Json(GetRolesOutput { roles }));
}
