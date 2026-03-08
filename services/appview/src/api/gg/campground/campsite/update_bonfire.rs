use appview_schema::{models::appview::Bonfire, schema::appview};
use campground_lexicon::gg::campground::campsite::BonfireViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::bonfire_view_basic, permissions::{CampsitePermissionConsts, TentPermissionConsts, has_tent_perms_or_owner}, ws::event_next_bonfire}, realtime::data::ReactiveSubject, util::params::{OptionValidity, ensure_valid_modified_uri}, xrpc::{
    campsite::BonfireInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateBonfireBody<'a> {
    name: Option<String>,
    description: Option<String>,
    avatar_uri: Option<&'a str>,
    banner_uri: Option<&'a str>,
}

#[post("/xrpc/gg.campground.campsite.updateBonfire?<bonfire_id>", data = "<body>")]
pub async fn update_bonfire<'a>(auth: BonfireInfo<'_>, event_subject: &State<ReactiveSubject>, bonfire_id: &str, body: Json<UpdateBonfireBody<'a>>) -> Result<Json<BonfireViewBasic>> {    
    let UpdateBonfireBody { name, description, avatar_uri, banner_uri } = &body.into_inner();
    let name = &name
        .clone()
        .ensure_validity(|x| x.len() >= 3 && x.len() <= 48)
        .map_err(|_|
            XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string())
        )?;
    let description = &description
        .clone()
        .ensure_validity(|x| x.len() <= 200)
        .map_err(|_|
            XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string())
        )?;
    let avatar_uri = &ensure_valid_modified_uri(avatar_uri)
        .map_err(|x|
            XRPCError::BadRequest(x.to_string())
        )?
        .map_passed(|x| x.to_string());
    let banner_uri = &ensure_valid_modified_uri(banner_uri)
        .map_err(|x|
            XRPCError::BadRequest(x.to_string())
        )?
        .map_passed(|x| x.to_string());

    let mut conn = establish_connection().unwrap();

    if !has_tent_perms_or_owner(&auth.campsite, &bonfire_id, None, None, &auth.member, CampsitePermissionConsts::MANAGE_BONFIRES, TentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    println!("Banner URI: {:?}", banner_uri);

    let current_date = Utc::now().naive_utc();

    let bonfire = diesel::update(appview::bonfire::table)
        .filter(
            appview::bonfire::campsiteid
                .eq(&auth.bonfire.campsite_id)
                .and(
                    appview::bonfire::id
                        .eq(bonfire_id)
                )
        )
        .set((
            appview::bonfire::name
                .eq(name.clone().unwrap_or(auth.bonfire.name)),
            appview::bonfire::description
                .eq(description.clone().unwrap_or(auth.bonfire.description)),
            appview::bonfire::avataruri
                .eq(avatar_uri.clone().with_fallback(auth.bonfire.avatar_uri)),
            appview::bonfire::banneruri
                .eq(banner_uri.clone().with_fallback(auth.bonfire.banner_uri)),
            appview::bonfire::updatedby
                .eq(&auth.actor.did),
            appview::bonfire::updatedat
                .eq(current_date),
        ))
        .load::<Bonfire>(&mut conn)
        .map_err(handle_select_first_error)?;

    let bonfire = bonfire.first().unwrap();

    event_next_bonfire(event_subject, &bonfire, false, "BonfireUpdated", bonfire_view_basic(bonfire));
    
    return Ok(Json(bonfire_view_basic(bonfire)));
}