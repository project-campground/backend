use appview_schema::models::appview::{Tent, TentCategory};
use campground_lexicon::gg::campground::tent::{
    GetTentsOutput, TentCategoryViewInList, TentType, TentViewInList,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;

use crate::{database::establish_connection, xrpc::error::Result};

pub async fn get_tents(campsite_id: &str, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
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
        .map(|tent_category| TentCategoryViewInList {
            id: tent_category.id,
            name: tent_category.name.clone(),
            description: tent_category.description.clone(),
            position: tent_category.priority,
            permissions: vec![],
        })
        .collect::<Vec<_>>();
    let tents = crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::campsiteid
                .eq(campsite_id)
                .and(crate::schema::appview::tent::bonfireid.eq(bonfire_id)),
        )
        .load::<Tent>(&mut conn)
        .expect("Error loading tents")
        .iter()
        .map(|tent| TentViewInList {
            id: tent.id.clone(),
            category_id: tent.category_id,
            name: tent.name.clone(),
            description: tent.description.clone(),
            r#type: TentType::Text,
            view_type: tent.view_type,
            position: tent.priority,
            permissions: vec![],
        })
        .collect::<Vec<_>>();

    return Ok(Json(GetTentsOutput { tents, categories }));
}
