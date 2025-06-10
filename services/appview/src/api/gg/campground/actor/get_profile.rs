use atproto_identity::{plc, storage_lru::LruDidDocumentStorage, web};
use campground_lexicon::gg::campground::actor::{Profile, ProfileViewDetailed};
use rocket::{serde::json::Json,State};
use common::record::fetch_record;
use reqwest::Client;

use crate::{
    auth_verifier::OptionalAuthorization, database::actors::get_actor, helpers::views::profile_view_detailed, xrpc_server::error::{Result, XRPCError}, DNS_RESOLVER
};

#[get("/xrpc/gg.campground.actor.getProfile?<actor>")]
pub async fn get_profile(_auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actor: &str) -> Result<Json<ProfileViewDetailed>> {
    let actor = get_actor(client, did_document_storage, actor).await.map_err(|_| XRPCError::NotFound)?;
    let did_doc = match *actor.did.split(":").collect::<Vec<&str>>().get(1).unwrap() {
        "plc" => {
            plc::query(client, "plc.directory", &actor.did).await.map_err(|_| XRPCError::NotFound)?
        },
        "web" => {
            web::query(client, &actor.did).await.map_err(|_| XRPCError::NotFound)?
        },
        _ => return Err(XRPCError::NotFound)
    };
    let response = fetch_record::<Profile>(
        &actor.did,
        "gg.campground.actor.profile",
        "self",
        client,
        did_document_storage,
        &DNS_RESOLVER
    ).await.map_err(|_| XRPCError::NotFound)?;
    return Ok(Json(profile_view_detailed(&did_doc, &response.value)))
}