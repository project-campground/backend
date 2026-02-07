use appview_schema::models::appview::{Campsite, CampsiteMember, CampsiteRole};
use campground_lexicon::gg::campground::campsite::CampsiteRoleViewBasic;
use chrono::Utc;
use diesel::RunQueryDsl;
use rocket::serde::json::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{campsites::get_roles_from_db, establish_connection}, helpers::{api::handle_select_first_error, campsites::campsite_role_view_basic, permissions::{CampsitePermissionConsts, aggregate_member_permissions}, roles::ensure_no_higher_role}, xrpc::{
    campsite::CampsiteInfo, error::{Result, XRPCError}
}};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateRoleBody {
    name: String,
    color: Option<i32>,
    color_secondary: Option<i32>,
    display_separately: Option<bool>,
    mentionable: Option<bool>,
    campsite_permissions: i64,
    tent_permissions: i64,
    priority: i32,
}

#[post("/xrpc/gg.campground.campsite.createRole?<campsite_id>", data = "<body>")]
pub async fn create_role(auth: CampsiteInfo<'_>, campsite_id: &str, body: Json<CreateRoleBody>) -> Result<Json<CampsiteRoleViewBasic>> {    
    let inner_body = &body.into_inner();
    if inner_body.name.len() == 0 || inner_body.name.len() > 64 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 1 to 64 characters".to_string()));
    }
    
    let mut conn = establish_connection().unwrap();

    let mut existing_roles = get_roles_from_db(&auth.campsite.id)?;

    if existing_roles.len() >= 150 {
        return Err(XRPCError::Forbidden("Cannot create more than 150 roles in a campsite".to_string()));
    }

    ensure_user_has_manage_role_permission(&auth.campsite, &auth.member, &existing_roles, inner_body.campsite_permissions, inner_body.tent_permissions)?;

    ensure_no_higher_role(auth.actor.did == auth.campsite.owner, &mut existing_roles, inner_body.priority, auth.member.roles.clone())?;

    let current_date = Utc::now().naive_utc();

    let role = &diesel::insert_into(crate::schema::appview::campsite_role::table)
        .values(
            CampsiteRole {
                id: Uuid::new_v4(),
                campsite_id: campsite_id.to_string(),
                name: inner_body.name.clone(),
                color: inner_body.color.clone().unwrap_or(0),
                color_secondary: inner_body.color_secondary.clone().unwrap_or(0),
                display_separately: inner_body.display_separately.clone().unwrap_or(false),
                mentionable: inner_body.mentionable.clone().unwrap_or(false),
                campsite_permissions: inner_body.campsite_permissions.clone(),
                tent_permissions: inner_body.tent_permissions.clone(),
                priority: inner_body.priority,
                created_by: auth.actor.did.clone(),
                created_at: current_date,
                updated_by: auth.actor.did,
                updated_at: current_date,
                flags: 0,
                members: vec![],
            }
        )
        .get_result::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    return Ok(Json(campsite_role_view_basic(role)));
}

fn ensure_user_has_manage_role_permission(campsite: &Campsite, member: &CampsiteMember, roles: &Vec<CampsiteRole>, given_campsite_permissions: i64, given_tent_permissions: i64) -> Result<(), XRPCError> {
    if campsite.owner == member.user_id {
        return Ok(());
    }

    let (campsite_permissions, tent_permissions) = aggregate_member_permissions(&member, &roles);

    // The user might not even have the permission to manage roles
    if campsite_permissions & CampsitePermissionConsts::MANAGE_ROLES == 0 {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    } else if (given_campsite_permissions & campsite_permissions != given_campsite_permissions) || (given_tent_permissions & tent_permissions != given_tent_permissions) {
        // Ensure user does not give themselves Manage Tent permission if they have Give Role & Manage Roles combo
        return Err(XRPCError::Forbidden("Cannot give role permissions that the member does not have".to_string()));
    }

    Ok(())
}