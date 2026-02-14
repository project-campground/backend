use appview_schema::{models::appview::Campsite, schema::appview};
use campground_lexicon::gg::campground::campsite::CampsiteViewBasic;
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_view_basic, permissions::{CampsitePermissionConsts, has_role_perms_or_owner}, ws::event_next_campsite_global}, realtime::data::ReactiveSubject, util::params::{AsParamValue, OptionValidity, ensure_valid_modified_uri}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateCampsiteBody<'a> {
    name: Option<&'a str>,
    description: Option<&'a str>,
    vanity_url: Option<&'a str>,
    tags: Option<Vec<&'a str>>,
    avatar_uri: Option<&'a str>,
    banner_uri: Option<&'a str>,
}

#[post("/xrpc/gg.campground.campsite.updateCampsite?<campsite_id>", data = "<body>")]
pub async fn update_campsite(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, body: Json<UpdateCampsiteBody<'_>>) -> Result<Json<CampsiteViewBasic>> {    
    let UpdateCampsiteBody { name, description, vanity_url, tags, avatar_uri, banner_uri } = &body.into_inner();
    let name = &name
        .ensure_validity(|x| x.len() >= 3 && x.len() <= 48)
        .map_err(|_|
            XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string())
        )?;
    let description = &description
        .ensure_validity(|x| x.len() <= 200)
        .map_err(|_|
            XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string())
        )?;
    let vanity_url = &vanity_url
        .as_value()
        .ensure_validity(|x| x.len() <= 32)
        .map_err(|_|
            XRPCError::BadRequest("Expected 'vanity_url' property to have a string of up to 32 characters".to_string())
        )?;
    let tags = &tags
        .clone()
        .ensure_validity(|x| x.len() <= 10 && !x.iter().any(|y| y.len() > 20))
        .map_err(|_|
            XRPCError::BadRequest("Expected 'tags' property to have up to 10 values and value to be a string of length up to 20 characters".to_string())
        )?
        .map(|x|
            x
                .iter()
                .map(|x| Some(x.to_string()))
                .collect::<Vec<Option<String>>>()
        );
    let avatar_uri = &ensure_valid_modified_uri(avatar_uri).map_err(|x|
        XRPCError::BadRequest(x.to_string())
    )?;
    let banner_uri = &ensure_valid_modified_uri(banner_uri).map_err(|x|
        XRPCError::BadRequest(x.to_string())
    )?;

    if !has_role_perms_or_owner(&auth.campsite, &auth.member, CampsitePermissionConsts::MANAGE_CAMPSITE, 0).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let current_date = Utc::now().naive_utc();

    let campsite = diesel::update(appview::campsite::table)
        .filter(
            appview::campsite::id
                .eq(campsite_id)
        )
        .set((
            appview::campsite::name
                .eq(name.unwrap_or(auth.campsite.name.as_str())),
            appview::campsite::description
                .eq(description.unwrap_or(auth.campsite.description.as_str())),
            appview::campsite::tags
                .eq(
                    tags.clone().unwrap_or(auth.campsite.tags)
                ),
            appview::campsite::vanityurl
                .eq(vanity_url.clone().with_fallback(auth.campsite.vanity_url.as_deref())),
            appview::campsite::avataruri
                .eq(avatar_uri.clone().with_fallback(auth.campsite.avatar_uri.as_ref().map(|x| x.as_str()))),
            appview::campsite::banneruri
                .eq(banner_uri.clone().with_fallback(auth.campsite.banner_uri.as_ref().map(|x| x.as_str()))),
            appview::campsite::updatedby
                .eq(&auth.actor.did),
            appview::campsite::updatedat
                .eq(current_date),
        ))
        .get_result::<Campsite>(&mut conn)
        .map_err(handle_select_first_error)?;

    event_next_campsite_global(event_subject, &auth.campsite.id, "CampsiteUpdated", campsite_view_basic(&campsite));

    return Ok(Json(campsite_view_basic(&campsite)));
}