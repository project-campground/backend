use campground_lexicon::gg::campground::actor::{Profile, GetProfilesOutput, ProfileViewDetailed};
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{
    auth_verifier::UserDidAuthOptional,
    helpers::{
        deduplicate_list, did::{try_get_did_doc, try_get_pds, try_resolve_at_identifier}, lower_list, repo::try_get_record, views::profile_view_detailed
    },
    xrpc_server::error::{Result, XRPCError},
    SharedIdResolver
};

#[get("/xrpc/gg.campground.actor.getProfiles?<actors>")]
pub async fn get_profiles(auth: UserDidAuthOptional, client: &State<Client>, id_resolver: &State<SharedIdResolver>, actors: Vec<&str>) -> Result<Json<GetProfilesOutput>> {
    let actors = deduplicate_list(lower_list(actors));
    if actors.len() > 25 || actors.len() == 0 {
        return Err(XRPCError::BadRequest);
    }
    let mut profile_views: Vec<ProfileViewDetailed> = vec![];
    let mut resolver = id_resolver.id_resolver.write().await;
    for actor in actors {
        let actor = try_resolve_at_identifier(&mut resolver, &actor.to_owned()).await?;
        let did_doc = try_get_did_doc(&mut resolver, &actor.to_owned()).await?;
        if did_doc.also_known_as.is_none() {
            return Err(XRPCError::NotFound);
        }
        let pds = try_get_pds(&did_doc)?;
        let response = try_get_record::<Profile>(client, &pds.service_endpoint, &actor, "gg.campground.actor.profile", "self").await?;
        profile_views.push(profile_view_detailed(&did_doc, &response.value));
    }

    return Ok(Json(GetProfilesOutput { profiles: profile_views }))
}