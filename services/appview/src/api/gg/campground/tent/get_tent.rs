use appview_schema::models::appview::Tent;
use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::tent::TentViewDetailed;
use diesel::{ExpressionMethods, RunQueryDsl, query_dsl::methods::FilterDsl};
use reqwest::Client;
use rocket::{State, serde::json::Json};
use uuid::Uuid;

use crate::{
    database::{actors::get_actor, establish_connection}, helpers::tents::tent_view_detailed, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.tent.getTent?<id>")]
pub async fn get_tent(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, id: &str) -> Result<Json<TentViewDetailed>> {
    let mut conn = establish_connection().unwrap();

    let actor_did = auth.1.jose.issuer.ok_or(XRPCError::Unauthorized)?;
    let actor = get_actor(client, did_document_storage, actor_did.as_str())
        .await
        .map_err(|_| XRPCError::Unauthorized)?;
    
    let uuid = Uuid::try_parse(id)
        .map_err(|_| XRPCError::BadRequest("Invalid 'id' format. Expected UUID".to_string()))?;

    let tent = &crate::schema::appview::tent::table
        .filter(
            crate::schema::appview::tent::id
                .eq(uuid)
        )
        .first::<Tent>(&mut conn)
        .map_err(|x|
            match x {
                diesel::result::Error::NotFound => XRPCError::NotFound,
                _ => XRPCError::InternalServerError,
            }
        )?;
    // Not in the campsite to view that
    if !actor.campsites.contains(&Some(tent.campsite_id.to_string())) {
        return Err(XRPCError::Forbidden("User cannot view campsite that they are not member of".to_string()));
    }

    let tent_view = tent_view_detailed(&tent);

    return Ok(Json(tent_view));
}