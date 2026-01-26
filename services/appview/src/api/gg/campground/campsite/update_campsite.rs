use appview_schema::{models::appview::Campsite, schema::appview};
use campground_lexicon::gg::campground::campsite::CampsiteViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_view_basic, permissions::{CampsitePermissionConsts, TentPermissionConsts, has_tent_perms_or_owner}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateCampsiteBody {
    name: Option<String>,
    description: Option<String>,
    vanity_url: Option<String>,
    tags: Option<Vec<String>>,
}

#[post("/xrpc/gg.campground.campsite.updateCampsite?<campsite_id>", data = "<body>")]
pub async fn update_campsite(auth: CampsiteInfo<'_>, campsite_id: &str, bonfire_id: &str, body: Json<UpdateCampsiteBody>) -> Result<Json<CampsiteViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    } else if inner_body.vanity_url.clone().map_or(false, |x| x.len() > 32) {
        return Err(XRPCError::BadRequest("Expected 'vanity_url' property to have a string of up to 32 characters".to_string()));
    } else if inner_body.tags.clone().map_or(false, |x| x.len() > 10 || x.iter().any(|y| y.len() > 20)) {
        return Err(XRPCError::BadRequest("Expected 'tags' property to have up to 10 values and value to be a string of length up to 20 characters".to_string()));
    }

    if !has_tent_perms_or_owner(auth.campsite.clone(), bonfire_id.to_string(), None, None, auth.member.clone(), CampsitePermissionConsts::MANAGE_CAMPSITE, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let new_vanity_url = inner_body.vanity_url.clone().map_or(auth.campsite.vanity_url, |x| if x == "" { None } else { Some(x) });
    let current_date = Utc::now().naive_utc();

    let (name, description, tags) = (
        inner_body.name.clone().unwrap_or(auth.campsite.name),
        inner_body.description.clone().unwrap_or(auth.campsite.description),
        inner_body.tags
            .clone()
            .map(|x|
                x
                    .iter()
                    .map(|y| Some(y.clone()))
                    .collect::<Vec<Option<String>>>()
            )
            .unwrap_or(auth.campsite.tags)
    );

    let campsite = diesel::update(appview::campsite::table)
        .filter(
            appview::campsite::id
                .eq(campsite_id)
        )
        .set((
            appview::campsite::name
                .eq(name),
            appview::campsite::description
                .eq(description),
            appview::campsite::tags
                .eq(tags),
            appview::campsite::vanityurl
                .eq(new_vanity_url),
            appview::campsite::updatedby
                .eq(auth.actor.did.clone()),
            appview::campsite::updatedat
                .eq(current_date),
        ))
        .get_result::<Campsite>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(campsite_view_basic(&campsite)));
}