use atproto_identity::{model::Document, plc, storage_lru::LruDidDocumentStorage, web};
use campground_lexicon::gg::campground::actor::{Profile, GetProfilesOutput, ProfileViewDetailed};
use common::fetch_record;
use rocket::{serde::json::Json,State};
use reqwest::Client;

use crate::{
    auth_verifier::OptionalAuthorization, database::actors::get_actor, helpers::{
        deduplicate_list, lower_list, views::profile_view_detailed
    }, xrpc_server::error::{Result, XRPCError}, DNS_RESOLVER
};

async fn get_profiles_inner(_auth: &OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actor: &String) -> Result<(Document, Profile)> {
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
    Ok((did_doc, response.value))
}

#[get("/xrpc/gg.campground.actor.getProfiles?<actors>")]
pub async fn get_profiles(auth: OptionalAuthorization, client: &State<Client>, did_document_storage: &State<LruDidDocumentStorage>, actors: Vec<&str>) -> Result<Json<GetProfilesOutput>> {
    let actors = deduplicate_list(lower_list(actors));
    if actors.len() > 25 || actors.len() == 0 {
        return Err(XRPCError::BadRequest);
    }
    let mut profile_views: Vec<ProfileViewDetailed> = vec![];
    for actor in actors {
        match get_profiles_inner(&auth, client, did_document_storage, &actor.to_owned()).await {
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