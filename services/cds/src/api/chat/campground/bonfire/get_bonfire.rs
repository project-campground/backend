use campground_lexicon::chat::campground::bonfire::BonfireViewDetailed;
use cds_schema::models::cds::Bonfire;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection,
    views::bonfires::bonfire_view_detailed,
    xrpc::{campsite::CampsiteInfoBasic, error::Result},
};

#[get("/xrpc/chat.campground.bonfire.getBonfire?<campsite_id>&<bonfire_id>")]
pub async fn get_bonfire(
    _auth: CampsiteInfoBasic<'_>,
    campsite_id: &str,
    bonfire_id: &str,
) -> Result<Json<BonfireViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let bonfire = &crate::schema::cds::bonfire::table
        .filter(
            crate::schema::cds::bonfire::campsiteid
                .eq(campsite_id)
                .and(crate::schema::cds::bonfire::id.eq(bonfire_id)),
        )
        .first::<Bonfire>(&mut conn)
        .expect("Error loading campsite's bonfire");

    let bonfire_view = bonfire_view_detailed(bonfire);

    return Ok(Json(bonfire_view));
}
