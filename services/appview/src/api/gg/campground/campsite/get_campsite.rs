use appview_schema::models::appview::{Bonfire, Campsite, CampsiteRole};
use campground_lexicon::gg::campground::campsite::{BonfireViewBasic, CampsiteRoleViewBasic, CampsiteViewDetailed};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::{bonfire_view_basic, campsite_role_view_basic, campsite_view_detailed}}, xrpc::{
    campsite::CampsiteInfoBasic, error::Result
}};

#[get("/xrpc/gg.campground.campsite.getCampsite?<campsite_id>")]
pub async fn get_campsite(_auth: CampsiteInfoBasic<'_>, campsite_id: &str) -> Result<Json<CampsiteViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let campsite = &crate::schema::appview::campsite::table
        .filter(crate::schema::appview::campsite::id.eq(campsite_id))
        .first::<Campsite>(&mut conn)
        .map_err(handle_select_first_error)?;
    let bonfires = crate::schema::appview::bonfire::table
        .filter(crate::schema::appview::bonfire::campsiteid.eq(campsite_id))
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(bonfire_view_basic)
        .collect::<Vec<BonfireViewBasic>>();
    let roles = crate::schema::appview::campsite_role::table
        .filter(crate::schema::appview::campsite_role::campsiteid.eq(campsite_id))
        .load::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?
        .iter()
        .map(campsite_role_view_basic)
        .collect::<Vec<CampsiteRoleViewBasic>>();
    let campsite_view = campsite_view_detailed(campsite, bonfires, roles);

    return Ok(Json(campsite_view));
}