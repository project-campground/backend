use appview_schema::models::appview::{Tent, TentCategory};
use campground_lexicon::gg::campground::tent::{GetTentsOutput, TentCategoryView, TentViewBasic};
use diesel::{BoolExpressionMethods, ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use rocket::serde::json::Json;

use crate::{
    database::establish_connection, helpers::tents::{tent_category_view, tent_view_basic}, xrpc::{
        auth::Authorization,
        error::Result
    }
};

#[get("/xrpc/gg.campground.tent.getTents?<campsite_id>&<bonfire_id>")]
pub async fn get_tents(_auth: Authorization<'_>, campsite_id: &str, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
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
        .expect("Error loading tents")
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
        .expect("Error loading tent categories")
        .iter()
        .map(tent_category_view)
        .collect::<Vec<TentCategoryView>>();

    return Ok(Json(GetTentsOutput { tents, categories }));
}