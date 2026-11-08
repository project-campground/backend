use appview_schema::models::appview::Bonfire;
use campground_lexicon::gg::campground::bonfire::BonfireViewDetailed;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    views::bonfires::bonfire_view_detailed,
    xrpc::{campsite::CampsiteInfoBasic, error::Result},
};

#[get("/xrpc/gg.campground.bonfire.getBonfire?<campsite_id>&<bonfire_id>")]
pub async fn get_bonfire(
    _auth: CampsiteInfoBasic<'_>,
    campsite_id: &str,
    bonfire_id: &str,
) -> Result<Json<BonfireViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let bonfire = &crate::schema::appview::bonfire::table
        .filter(
            crate::schema::appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::bonfire::id.eq(bonfire_id)),
        )
        .first::<Bonfire>(&mut conn)
        .expect("Error loading campsite's bonfire");

    let bonfire_view = bonfire_view_detailed(bonfire);

    return Ok(Json(bonfire_view));
}
