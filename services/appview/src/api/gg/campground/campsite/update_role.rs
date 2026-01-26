use appview_schema::{models::appview::CampsiteRole, schema::appview::campsite_role};
use campground_lexicon::gg::campground::campsite::CampsiteRoleViewBasic;
use chrono::Utc;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::establish_connection, helpers::{api::handle_select_first_error, campsites::campsite_role_view_basic}, xrpc::{
    campsite::CampsiteInfoBasic, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct UpdateRoleBody {
    name: Option<String>,
    color: Option<i32>,
    color_secondary: Option<i32>,
    display_separately: Option<bool>,
    mentionable: Option<bool>,
    campsite_permissions: Option<i64>,
    tent_permissions: Option<i64>,
    priority: Option<i32>,
}

#[post("/xrpc/gg.campground.campsite.updateRole?<campsite_id>&<role_id>", data = "<body>")]
pub async fn update_role(auth: CampsiteInfoBasic<'_>, campsite_id: &str, role_id: &str, body: Json<UpdateRoleBody>) -> Result<Json<CampsiteRoleViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() == 0 || x.len() > 64) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 1 to 64 characters".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    
    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))
        ?;
    
    let role = &campsite_role::table
        .filter(campsite_role::id.eq(role_id_uuid).and(campsite_role::campsiteid.eq(campsite_id)))
        .first::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    let current_date = Utc::now().naive_utc();

    let updated_role = &diesel::update(campsite_role::table)
        .filter(
            campsite_role::id.eq(role.id)
        )
        .set((
            // Mandatory
            campsite_role::updatedby
                .eq(&auth.actor.did),
            campsite_role::updatedat
                .eq(current_date),
            // Stuff changed
            campsite_role::name
                .eq(inner_body.name.clone().unwrap_or(role.name.clone())),
            campsite_role::priority
                .eq(inner_body.priority.clone().unwrap_or(role.priority)),
            campsite_role::color
                .eq(inner_body.color.clone().unwrap_or(role.color)),
            campsite_role::displayseparately
                .eq(inner_body.display_separately.clone().unwrap_or(role.display_separately)),
            campsite_role::mentionable
                .eq(inner_body.mentionable.clone().unwrap_or(role.mentionable)),
            campsite_role::colorsecondary
                .eq(inner_body.color_secondary.clone().unwrap_or(role.color_secondary)),
            campsite_role::tentpermissions
                .eq(inner_body.tent_permissions.clone().unwrap_or(role.tent_permissions)),
            campsite_role::campsitepermissions
                .eq(inner_body.campsite_permissions.clone().unwrap_or(role.campsite_permissions)),
        ))
        .get_result::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(campsite_role_view_basic(updated_role)));
}