use campground_lexicon::chat::campground::permission::{
    PermissionViewDetailed, PermissionViewEmbedded,
};
use cds_schema::models::cds::{Actor, CampsiteMember, Role, Permission, Profile};

use crate::views::{
    members::member_view_author, profiles::profile_view_basic, roles::role_view_basic,
    util::serialize_datetime,
};

pub fn permission_view_embedded(permission: &Permission) -> PermissionViewEmbedded {
    return PermissionViewEmbedded {
        user_id: permission.actor.clone(),
        role_id: permission.role.clone(),

        permissions: (permission.permissions as u64).to_string(),
    };
}

pub fn permission_view_detailed(
    permission: &Permission,
    role: Option<&Role>,
    member: Option<(&Actor, Option<&Profile>, Option<&CampsiteMember>)>,
) -> PermissionViewDetailed {
    return PermissionViewDetailed {
        bonfire_id: permission.bonfire.clone(),
        category_id: permission.category.clone(),
        tent_id: permission.tent.clone(),

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
