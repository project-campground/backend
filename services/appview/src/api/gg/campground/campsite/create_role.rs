use appview_schema::models::appview::{Campsite, CampsiteMember, CampsiteRole};
use campground_lexicon::gg::campground::campsite::CampsiteRoleViewBasic;
use chrono::Utc;
use diesel::RunQueryDsl;
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::{campsites::get_roles_from_db, establish_connection}, helpers::{api::handle_select_first_error, campsites::campsite_role_view_basic, permissions::{CampsitePermissionConsts, aggregate_member_permissions}, ws::event_next_campsite}, realtime::data::ReactiveSubject, xrpc::{
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
}

#[post("/xrpc/gg.campground.campsite.createRole?<campsite_id>", data = "<body>")]
pub async fn create_role(auth: CampsiteInfo<'_>, event_subject: &State<ReactiveSubject>, campsite_id: &str, body: Json<CreateRoleBody>) -> Result<Json<CampsiteRoleViewBasic>> {    
    let CreateRoleBody { name, color, color_secondary, display_separately, mentionable, campsite_permissions, tent_permissions } = &body.into_inner();
    if name.len() == 0 || name.len() > 64 {
        return Err(XRPCError::BadRequest("Expected 'name' property to have a string of length 1 to 64 characters".to_string()));
    }
    
    let mut conn = establish_connection().unwrap();

    let existing_roles = get_roles_from_db(&auth.campsite.id)?;

    if existing_roles.len() >= 150 {
        return Err(XRPCError::Forbidden("Cannot create more than 150 roles in a campsite".to_string()));
    }

    ensure_user_has_manage_role_permission(&auth.campsite, &auth.member, &existing_roles, *campsite_permissions, *tent_permissions)?;

    let lowest_priority = existing_roles
        .iter()
        .max_by(|x, y| x.priority.cmp(&y.priority))
        .unwrap();

    let current_date = Utc::now().naive_utc();

    let role = &diesel::insert_into(crate::schema::appview::campsite_role::table)
        .values(
            CampsiteRole {
                id: Uuid::new_v4(),
                campsite_id: campsite_id.to_string(),
                name: name.clone(),
                color: color.clone().unwrap_or(0),
                color_secondary: color_secondary.clone().unwrap_or(0),
                display_separately: display_separately.clone().unwrap_or(false),
                mentionable: mentionable.clone().unwrap_or(false),
                campsite_permissions: *campsite_permissions,
                tent_permissions: *tent_permissions,
                priority: lowest_priority.priority + 1,
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

    event_next_campsite(event_subject, &auth.campsite.id, 0, "RoleCreated", campsite_role_view_basic(role));

    return Ok(Json(campsite_role_view_basic(role)));
}

fn ensure_user_has_manage_role_permission(campsite: &Campsite, member: &CampsiteMember, roles: &Vec<CampsiteRole>, given_campsite_permissions: i64, given_tent_permissions: i64) -> Result<(), XRPCError> {
    if campsite.owner == member.user_id {
        return Ok(());
    }

    let permissions = aggregate_member_permissions(&member, &roles);

    // The user might not even have the permission to manage roles
    if permissions.campsite & CampsitePermissionConsts::MANAGE_ROLES == 0 {
        return Err(XRPCError::Forbidden("No given permission to do that".to_string()));
    } else if (given_campsite_permissions & permissions.campsite != given_campsite_permissions) || (given_tent_permissions & permissions.campsite != given_tent_permissions) {
        // Ensure user does not give themselves Manage Tent permission if they have Give Role & Manage Roles combo
        return Err(XRPCError::Forbidden("Cannot give role permissions that the member does not have".to_string()));
    }

    Ok(())
}