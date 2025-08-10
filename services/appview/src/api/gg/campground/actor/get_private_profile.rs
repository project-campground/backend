use atproto_identity::storage_lru::LruDidDocumentStorage;
use campground_lexicon::gg::campground::actor::PrivateProfileView;
use rocket::{serde::json::Json, State};
use reqwest::Client;

use crate::{
    config::CORE_CONFIG, database::{actors::get_actor, private_profiles}, helpers::views::private_profile_view, xrpc::{
        auth::Authorization,
        error::{Result, XRPCError}
    }
};

#[get("/xrpc/gg.campground.actor.getPrivateProfile?<actor>&<for_actor>")]
pub async fn get_private_profile(auth: Authorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actor: &str, for_actor: &str) -> Result<Json<PrivateProfileView>> {
    let actor = get_actor(client, did_document_storage, actor).await.map_err(|_| XRPCError::NotFound)?;
    let for_actor = get_actor(client, did_document_storage, for_actor).await.map_err(|_| XRPCError::NotFound)?;

    let issuer = match auth.1.jose.issuer {
        Some(issuer) => issuer,
        None => return Err(XRPCError::Unauthorized)
    };
    if for_actor.did == CORE_CONFIG.did() {
        // this endpoint should never get called by a user on the same home server
        // as users on the same server will be handled by the getProfile endpoint
        return Err(XRPCError::BadRequest);
    }
    if actor.home_server != CORE_CONFIG.did() {
        return Err(XRPCError::BadRequest);
    }
    if issuer != for_actor.did {
        return Err(XRPCError::Unauthorized);
    }

    let account = private_profiles::get_account(client, did_document_storage, &actor.did).await.map_err(|_| XRPCError::NotFound)?;

    Ok(Json(private_profile_view(&account)))
}