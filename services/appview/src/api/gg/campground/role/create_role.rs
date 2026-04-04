use appview_schema::models::appview::{Campsite, CampsiteMember, CampsiteRole};
use campground_lexicon::gg::campground::{
    permission::PermissionsDictionary,
    role::{RoleMotion, RoleViewBasic},
};
use chrono::Utc;
use diesel::RunQueryDsl;
use rocket::{State, serde::json::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{campsites::get_roles_from_db, establish_connection},
    helpers::{
        api::handle_select_first_error,
        permissions::{GeneralPermissionConsts, aggregate_member_permissions},
        ws::event_next_campsite,
    },
    realtime::data::ReactiveSubject,
    views::roles::{from_role_motion, role_view_basic},
    xrpc::{
        campsite::CampsiteInfo,
        error::{Result, XRPCError},
    },
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct CreateRoleBody {
    name: String,
    motion: Option<RoleMotion>,
    colors: Option<Vec<u32>>,
    display_separately: Option<bool>,
    mentionable: Option<bool>,
    permissions: PermissionsDictionary,
}

#[post("/xrpc/gg.campground.role.createRole?<campsite_id>", data = "<body>")]
pub async fn create_role(
    auth: CampsiteInfo<'_>,
    event_subject: &State<ReactiveSubject>,
    campsite_id: &str,
    body: Json<CreateRoleBody>,
) -> Result<Json<RoleViewBasic>> {
    let CreateRoleBody {
        name,
        motion,
        colors,
        display_separately,
        mentionable,
        permissions,
    } = &body.into_inner();
    if name.len() == 0 || name.len() > 64 {
        return Err(XRPCError::BadRequest(
            "Expected 'name' property to have a string of length 1 to 64 characters".to_string(),
        ));
    } else if colors.clone().map_or(false, |colors| colors.len() > 5) {
        return Err(XRPCError::BadRequest(
            "Expected 'colors' property to be an array with max length of 5".to_string(),
        ));
    }

    let mut conn = establish_connection().unwrap();

    let existing_roles = get_roles_from_db(&auth.campsite.id)?;

    if existing_roles.len() >= 100 {
        return Err(XRPCError::Forbidden(
            "Cannot create more than 100 roles in a campsite".to_string(),
        ));
    }

    ensure_user_has_manage_role_permission(
        &auth.campsite,
        &auth.member,
        &existing_roles,
        permissions,
    )?;

    let lowest_priority = existing_roles
        .iter()
        .max_by(|x, y| x.priority.cmp(&y.priority))
        .unwrap();

    let current_date = Utc::now().naive_utc();

    let role = &diesel::insert_into(crate::schema::appview::campsite_role::table)
        .values(CampsiteRole {
            id: Uuid::new_v4(),
            campsite_id: campsite_id.to_string(),
            name: name.clone(),
            colors: colors.clone().map_or(vec![], |colors| {
                colors
                    .iter()
                    .map(|&color| Some(color as i32))
                    .collect::<Vec<Option<i32>>>()
            }),
            motion: motion.clone().map_or(0, |motion| from_role_motion(motion)),
            display_separately: display_separately.clone().unwrap_or(false),
            mentionable: mentionable.clone().unwrap_or(false),
            general_permissions: permissions.general,
            content_permissions: permissions.content,
            priority: lowest_priority.priority + 1,
            created_by: auth.actor.did.clone(),
            created_at: current_date,
            updated_by: auth.actor.did,
            updated_at: current_date,
            flags: 0,
            members: vec![],
        })
        .get_result::<CampsiteRole>(&mut conn)
        .map_err(handle_select_first_error)?;

    event_next_campsite(
        event_subject,
        &auth.campsite.id,
        0,
        "RoleCreated",
        role_view_basic(role),
    );

    return Ok(Json(role_view_basic(role)));
}

fn ensure_user_has_manage_role_permission(
    campsite: &Campsite,
    member: &CampsiteMember,
    roles: &Vec<CampsiteRole>,
    given_permissions: &PermissionsDictionary,
) -> Result<(), XRPCError> {
    if campsite.owner == member.user_id {
        return Ok(());
    }

    let permissions = aggregate_member_permissions(&member, &roles);

    // The user might not even have the permission to manage roles
    if permissions.general & GeneralPermissionConsts::MANAGE_ROLES == 0 {
        return Err(XRPCError::Forbidden(
            "No given permission to do that".to_string(),
        ));
    } else if (given_permissions.general & permissions.general != given_permissions.general)
        || (given_permissions.general & permissions.general != given_permissions.general)
    {
        // Ensure user does not give themselves Manage Tent permission if they have Give Role & Manage Roles combo
        return Err(XRPCError::Forbidden(
            "Cannot give role permissions that the member does not have".to_string(),
        ));
    }

    Ok(())
}
