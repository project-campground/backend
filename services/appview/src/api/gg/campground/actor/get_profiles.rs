use campground_lexicon::gg::campground::actor::{Profile, GetProfilesOutput, ProfileViewDetailed};
use rocket::{serde::json::Json,State};
use reqwest::Client;
use rsky_identity::{types::DidDocument, IdResolver};
use tokio::sync::RwLockWriteGuard;

use crate::{
    auth_verifier::UserDidAuthOptional,
    helpers::{
        deduplicate_list, did::{try_get_did_doc, try_get_pds, try_resolve_at_identifier}, lower_list, repo::try_get_record, views::profile_view_detailed
    },
    xrpc_server::error::{Result, XRPCError},
    SharedIdResolver
};

async fn get_profiles_inner(client: &State<Client>, resolver: &mut RwLockWriteGuard<'_, IdResolver>, actor: &String) -> Result<(DidDocument, Profile)> {
    let actor = try_resolve_at_identifier(resolver, &actor.to_owned()).await?;
    let did_doc = try_get_did_doc(resolver, &actor.to_owned()).await?;
    if did_doc.also_known_as.is_none() {
        return Err(XRPCError::NotFound);
    }
    let pds = try_get_pds(&did_doc)?;
    let response = try_get_record::<Profile>(client, &pds.service_endpoint, &actor, "gg.campground.actor.profile", "self").await?;
    Ok((did_doc, response.value))
}

#[get("/xrpc/gg.campground.actor.getProfiles?<actors>")]
pub async fn get_profiles(auth: UserDidAuthOptional, client: &State<Client>, id_resolver: &State<SharedIdResolver>, actors: Vec<&str>) -> Result<Json<GetProfilesOutput>> {
    let actors = deduplicate_list(lower_list(actors));
    if actors.len() > 25 || actors.len() == 0 {
        return Err(XRPCError::BadRequest);
    }
    let mut profile_views: Vec<ProfileViewDetailed> = vec![];
    let mut resolver = id_resolver.id_resolver.write().await;
    for actor in actors {
        match get_profiles_inner(client, &mut resolver, &actor.to_owned()).await {
            Ok((did_doc, profile)) => {
                profile_views.push(profile_view_detailed(&did_doc, &profile));
            },
            Err(_) => ()
        }
    }

    if profile_views.len() == 0 {
        return Err(XRPCError::BadRequest);
    }

    return Ok(Json(GetProfilesOutput { profiles: profile_views }))
}