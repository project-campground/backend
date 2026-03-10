#![
    allow(unused_variables)
]
use appview_schema::{models::appview::Tent, schema::appview};
use campground_lexicon::gg::campground::tent::TentViewBasic;
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::{State, serde::json::Json};
use serde::Deserialize;

use crate::{database::establish_connection, helpers::{permissions::{GeneralPermissionConsts, ContentPermissionConsts, has_leveled_perms_or_owner}, tents::tent_view_basic, ws::event_next_tent}, realtime::data::ReactiveSubject, xrpc::{
    campsite::TentInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateTentBody {
    name: Option<String>,
    description: Option<String>,
    view_type: Option<i16>,
}

#[post("/xrpc/gg.campground.tent.updateTent?<tent_id>", data = "<body>")]
pub async fn update_tent(auth: TentInfo<'_>, event_subject: &State<ReactiveSubject>, tent_id: &str, body: Json<UpdateTentBody>) -> Result<Json<TentViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.is_none() && inner_body.description.is_none() && inner_body.view_type.is_none() {
        return Err(XRPCError::BadRequest("Expected at least one property in the body".to_string()));
    } else if inner_body.name.clone().map_or(false, |x| x.len() < 3 || x.len() > 48) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 3 to 48 characters".to_string()));
    } else if inner_body.description.clone().map_or(false, |x| x.len() > 200) {
        return Err(XRPCError::BadRequest("Expected 'description' property to have a string of up to 200 characters".to_string()));
    } else if inner_body.view_type.clone().map_or(false, |x| x != 0) {
        return Err(XRPCError::BadRequest("Expected 'view_type' property to be 0".to_string()));
    }

    if !has_leveled_perms_or_owner(&auth.campsite, &auth.tent.bonfire_id, auth.tent.category_id.clone(), Some(auth.tent.id), &auth.member, GeneralPermissionConsts::MANAGE_TENTS, ContentPermissionConsts::VIEW_CONTENT).await? {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    }

    let mut conn = establish_connection().unwrap();

    let current_date = Utc::now().naive_utc();
    
    let updated_tents = diesel::update(crate::schema::appview::tent::table)
        .filter(
            appview::tent::id
                .eq(
                    auth.tent.id.clone()
                )
        )
        .set((
            // All the new settings
            appview::tent::name
                .eq(inner_body.name.clone().unwrap_or(auth.tent.name.clone())),
            appview::tent::description
                .eq(inner_body.description.clone().unwrap_or(auth.tent.description.clone())),
            appview::tent::viewtype
                .eq(inner_body.view_type.clone().unwrap_or(auth.tent.view_type)),
            // Mandatory
            appview::tent::updatedat
                .eq(current_date),
            appview::tent::updatedby
                .eq(&auth.actor.did),
        ))
        .load::<Tent>(&mut conn)
        .expect("Error updating tent");

    let first_tent = updated_tents.first().unwrap();

    event_next_tent(event_subject, &first_tent, false, "TentUpdated", tent_view_basic(&first_tent));

    return Ok(Json(tent_view_basic(first_tent)));
}
