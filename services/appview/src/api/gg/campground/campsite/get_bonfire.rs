use appview_schema::models::appview::{Bonfire, Tent, TentCategory};
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::{campsite::BonfireViewDetailed, tent::{TentCategoryView, TentViewBasic}};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{database::{actors::get_actor, establish_connection}, helpers::{campsites::bonfire_view_detailed, tents::{tent_category_view, tent_view_basic}}, xrpc::{
    auth::Authorization,
    error::{Result, XRPCError}
}};

#[get("/xrpc/gg.campground.campsite.getBonfire?<campsite_id>&<id>")]
pub async fn get_bonfire(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, campsite_id: &str, id: &str) -> Result<Json<BonfireViewDetailed>> {
    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?;

    let mut conn = establish_connection().unwrap();
    let actor = &get_actor(client, did_document_storage, actor_did.as_str())
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
                        .eq(id)
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
                        .eq(id)
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
                        .eq(id)
                )
        )
        .first::<Bonfire>(&mut conn)
        .expect("Error loading campsite's bonfire");

    let bonfire_view = bonfire_view_detailed(bonfire, tents, categories);

    return Ok(Json(bonfire_view));
}