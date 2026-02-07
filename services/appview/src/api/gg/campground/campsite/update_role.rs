use appview_schema::{models::appview::{Campsite, CampsiteMember, CampsiteRole}, schema::appview::campsite_role};
use campground_lexicon::gg::campground::campsite::CampsiteRoleViewBasic;
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{campsites::get_roles_from_db, establish_connection}, helpers::{api::handle_select_first_error, campsites::campsite_role_view_basic, permissions::{CampsitePermissionConsts, aggregate_member_permissions}}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
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

#[allow(unused_variables)]
#[post("/xrpc/gg.campground.campsite.updateRole?<campsite_id>&<role_id>", data = "<body>")]
pub async fn update_role(auth: CampsiteInfo<'_>, campsite_id: &str, role_id: &str, body: Json<UpdateRoleBody>) -> Result<Json<CampsiteRoleViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.clone().map_or(false, |x| x.len() == 0 || x.len() > 64) {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 1 to 64 characters".to_string()));
    }

    let mut conn = establish_connection().unwrap();
    
    // Can be given invalid UUID; Be descriptive
    let role_id_uuid = Uuid::try_parse(role_id)
        .map_err(|_| XRPCError::BadRequest("Expected 'id' query to be a valid UUID".to_string()))
        ?;

    let roles = &get_roles_from_db(&auth.campsite.id)?;
    let role = roles
        .iter()
        .find(|x| x.id == role_id_uuid)
        .ok_or(XRPCError::NotFound)?;

    ensure_user_has_manage_role_permission(&auth.campsite, &auth.member, roles, inner_body.campsite_permissions, inner_body.tent_permissions)?;

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


fn ensure_user_has_manage_role_permission(campsite: &Campsite, member: &CampsiteMember, roles: &Vec<CampsiteRole>, given_campsite_permissions: Option<i64>, given_tent_permissions: Option<i64>) -> Result<(), XRPCError> {
    if campsite.owner == member.user_id {
        return Ok(());
    }

    let (campsite_permissions, tent_permissions) = aggregate_member_permissions(&member, &roles);

    // The user might not even have the permission to manage roles
    if campsite_permissions & CampsitePermissionConsts::MANAGE_ROLES == 0 {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    } else if given_campsite_permissions.map_or(false, |x| x & campsite_permissions != x) && given_tent_permissions.map_or(false, |x| x & tent_permissions != x) {
        // Ensure user does not give themselves Manage Tent permission if they have Give Role & Manage Roles combo
        return Err(XRPCError::Forbidden("Cannot give role permissions that the member does not have".to_string()));
    }

    Ok(())
}