#![allow(dead_code)]
use appview_schema::models::appview::CampsiteRole;
use campground_lexicon::gg::campground::{
    permission::PermissionsDictionary,
    role::{RoleMotion, RoleViewBasic},
};
use uuid::Uuid;

use crate::{views::util::serialize_datetime, xrpc::error::XRPCError};

pub struct CampsiteRoleFlag {}
impl CampsiteRoleFlag {
    pub const DEFAULT_ROLE: i32 = 0b1;
}

pub fn role_view_basic(role: &CampsiteRole) -> RoleViewBasic {
    return RoleViewBasic {
        id: role.id.clone(),
        campsite_id: role.campsite_id.clone(),
        name: role.name.clone(),
        raised: role.display_separately.clone(),
        pingable: role.mentionable.clone(),
        colors: role
            .colors
            .iter()
            .filter_map(|color| color.map(|color| color as u32))
            .collect::<Vec<u32>>(),
        motion: to_role_motion(role.motion),
        permissions: PermissionsDictionary {
            general: role.general_permissions as u64,
            content: role.content_permissions as u64,
        },
        position: role.priority,
        created_by: role.created_by.clone(),
        created_at: serialize_datetime(role.created_at),
        updated_by: role.updated_by.clone(),
        updated_at: serialize_datetime(role.updated_at),
        flags: role.flags,
    };
}

pub fn to_role_motion(motion: i16) -> RoleMotion {
    match motion {
        1 => RoleMotion::Linear,
        2 => RoleMotion::Wave,
        3 => RoleMotion::Radial,
        _ => RoleMotion::None,
    }
}

pub fn from_role_motion(motion: RoleMotion) -> i16 {
    match motion {
        RoleMotion::Linear => 1,
        RoleMotion::Wave => 2,
        RoleMotion::Radial => 3,
        _ => 0,
    }
}

pub fn ensure_no_higher_role(
    is_owner: bool,
    all_roles: &mut Vec<CampsiteRole>,
    given_role_priority: i32,
    member_roles: Vec<Option<Uuid>>,
) -> Result<(), XRPCError> {
    if is_owner {
        return Ok(());
    }

    // Can't give role higher than they have or the same priority
    all_roles.sort_by(|a, b| a.priority.cmp(&b.priority));

    let highest_role = all_roles
        .iter()
        .find(|x| member_roles.contains(&Some(x.id)));

    return if highest_role.map_or(false, |x| x.priority <= given_role_priority) {
        Err(XRPCError::Forbidden(
            "The given role is higher or the same priority as your highest role".to_string(),
        ))
    } else {
        Ok(())
    };
}
