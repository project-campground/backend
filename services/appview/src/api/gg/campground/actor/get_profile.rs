use campground_lexicon::gg::campground::actor::{Profile, ProfileViewDetailed};
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{
    auth_verifier::UserDidAuthOptional,
    helpers::{
        did::{try_get_did_doc, try_get_pds, try_resolve_at_identifier},
        repo::try_get_record, views::profile_view_detailed
    },
    xrpc_server::error::{Result, XRPCError},
    SharedIdResolver
};

#[get("/xrpc/gg.campground.actor.getProfile?<actor>")]
pub async fn get_profile(auth: UserDidAuthOptional, client: &State<Client>, id_resolver: &State<SharedIdResolver>, actor: &str) -> Result<Json<ProfileViewDetailed>> {
    let actor = try_resolve_at_identifier(&mut id_resolver.id_resolver.write().await, &actor.to_owned()).await?;
    let did_doc = try_get_did_doc(&mut id_resolver.id_resolver.write().await, &actor.to_owned()).await?;
    if did_doc.also_known_as.is_none() {
        return Err(XRPCError::NotFound);
    }
    let pds = try_get_pds(&did_doc)?;
    let response = try_get_record::<Profile>(client, &pds.service_endpoint, &actor, "gg.campground.actor.profile", "self").await?;
    return Ok(Json(profile_view_detailed(&did_doc, &response.value)))
}