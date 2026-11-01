use appview_schema::models::appview::{
    Actor, CampsiteMember, CampsitePermission, CampsiteRole, Profile,
};
use campground_lexicon::gg::campground::permission::{PermissionViewBasic, PermissionViewDetailed};

use crate::views::{
    members::member_view_author, profiles::profile_view_basic, roles::role_view_basic,
    util::serialize_datetime,
};

pub fn permission_view_basic(permission: &CampsitePermission) -> PermissionViewBasic {
    return PermissionViewBasic {
        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        user_id: permission.user_id.clone(),
        role_id: permission.role_id.clone(),

        permissions: (permission.permissions as u64).to_string(),
    };
}

pub fn permission_view_detailed(
    permission: &CampsitePermission,
    role: Option<&CampsiteRole>,
    member: Option<(&Actor, Option<&Profile>, Option<&CampsiteMember>)>,
) -> PermissionViewDetailed {
    return PermissionViewDetailed {
        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        member: member
            .map(|member| member_view_author(member.2, profile_view_basic(member.0, member.1))),
        role: role.map(|role| role_view_basic(role)),

        permissions: (permission.permissions as u64).to_string(),

        created_by: permission.created_by.clone(),
        created_at: serialize_datetime(permission.created_at),
        updated_by: permission.updated_by.clone(),
        updated_at: serialize_datetime(permission.updated_at),
    };
}
