use appview_schema::models::appview::{Tent, TentCategory};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::{GetTentsOutput, TentCategoryView, TentViewBasic};
use diesel::{BoolExpressionMethods, ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};

use crate::{
    database::{actors::get_actor, establish_connection}, helpers::tents::{tent_category_view, tent_view_basic}, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.tent.getTents?<campsite_id>&<bonfire_id>")]
pub async fn get_tents(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, campsite_id: &str, bonfire_id: &str) -> Result<Json<GetTentsOutput>> {
    let mut conn = establish_connection().unwrap();

    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?;
    let actor = get_actor(client, did_document_storage, actor_did.as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;
    // Not in the campsite to view that
    if !actor.campsites.contains(&Some(campsite_id.to_string())) {
        return Err(XRPCError::Forbidden("User cannot view campsite that they are not member of".to_string()));
    }

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