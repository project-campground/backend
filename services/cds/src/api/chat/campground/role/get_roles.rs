use campground_lexicon::gg::campground::role::{GetRolesOutput, RoleViewBasic};
use cds_schema::models::cds::CampsiteRole;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    helpers::api::handle_select_first_error,
    views::roles::role_view_basic,
    xrpc::{campsite::CampsiteInfoBasic, error::Result},
};

#[get("/xrpc/chat.campground.role.getRoles?<campsite_id>")]
pub async fn get_roles(
    _auth: CampsiteInfoBasic<'_>,
    campsite_id: &str,
) -> Result<Json<GetRolesOutput>> {
    let mut conn = establish_connection().unwrap();

    let roles = crate::schema::cds::campsite_role::table
        .filter(crate::schema::cds::campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(role_view_basic)
        .collect::<Vec<RoleViewBasic>>();

    return Ok(Json(GetRolesOutput { roles }));
}
