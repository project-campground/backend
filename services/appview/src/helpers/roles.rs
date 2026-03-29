#![
    allow(dead_code)
]
use appview_schema::models::appview::CampsiteRole;
use campground_lexicon::gg::campground::campsite::CampsiteRoleMotion;
use uuid::Uuid;

use crate::xrpc::error::XRPCError;

pub struct CampsiteRoleFlag { }
impl CampsiteRoleFlag {
    pub const DEFAULT_ROLE: i32 = 0b1;
}

pub fn to_role_motion(motion: i16) -> CampsiteRoleMotion {
    match motion {
        1 => CampsiteRoleMotion::Linear,
        2 => CampsiteRoleMotion::Wave,
        3 => CampsiteRoleMotion::Radial,
        _ => CampsiteRoleMotion::None,
    }
}

pub fn from_role_motion(motion: CampsiteRoleMotion) -> i16 {
    match motion {
        CampsiteRoleMotion::Linear => 1,
        CampsiteRoleMotion::Wave => 2,
        CampsiteRoleMotion::Radial => 3,
        _ => 0,
    }
}

pub fn ensure_no_higher_role(is_owner: bool, all_roles: &mut Vec<CampsiteRole>, given_role_priority: i32, member_roles: Vec<Option<Uuid>>) -> Result<(), XRPCError> {
    if is_owner {
        return Ok(());
    }

    // Can't give role higher than they have or the same priority
    all_roles.sort_by(|a, b| a.priority.cmp(&b.priority));

    let highest_role = all_roles
        .iter()
        .find(|x| member_roles.contains(&Some(x.id)));

    return if highest_role.map_or(false, |x| x.priority <= given_role_priority) {
        Err(XRPCError::Forbidden("The given role is higher or the same priority as your highest role".to_string()))
    } else {
        Ok(())
    }
}
