use appview_schema::models::appview::{Bonfire, Tent, TentCategory};
use campground_lexicon::gg::campground::{campsite::BonfireViewDetailed, tent::{TentCategoryView, TentViewBasic}};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{database::establish_connection, helpers::{campsites::bonfire_view_detailed, tents::{tent_category_view, tent_view_basic}}, xrpc::{
    campsite::CampsiteInfoBasic, error::Result
}};

#[get("/xrpc/gg.campground.campsite.getBonfire?<campsite_id>&<bonfire_id>")]
pub async fn get_bonfire(_auth: CampsiteInfoBasic<'_>, campsite_id: &str, bonfire_id: &str) -> Result<Json<BonfireViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let tents = crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::tent::bonfireid
                        .eq(bonfire_id)
                )
        )
        .load::<Tent>(&mut conn)
        .expect("Error loading campsite's tents")
        .iter()
        .map(tent_view_basic)
        .collect::<Vec<TentViewBasic>>();
    let categories = crate::schema::appview::tent_category::table
        .filter(
            crate::schema::appview::tent_category::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::tent_category::bonfireid
                        .eq(bonfire_id)
                )
        )
        .load::<TentCategory>(&mut conn)
        .expect("Error loading campsite's tent categories")
        .iter()
        .map(tent_category_view)
        .collect::<Vec<TentCategoryView>>();
    let bonfire = &crate::schema::appview::bonfire::table
        .filter(
            crate::schema::appview::bonfire::campsiteid
                .eq(campsite_id)
                .and(
                    crate::schema::appview::bonfire::id
                        .eq(bonfire_id)
                )
        )
        .first::<Bonfire>(&mut conn)
        .expect("Error loading campsite's bonfire");

    let bonfire_view = bonfire_view_detailed(bonfire, tents, categories);

    return Ok(Json(bonfire_view));
}