use appview_schema::models::appview::CampsitePermission;
use campground_lexicon::gg::campground::permission::{PermissionViewBasic, PermissionViewDetailed};

use crate::views::util::serialize_datetime;

pub fn permission_view_basic(permission: &CampsitePermission) -> PermissionViewBasic {
    return PermissionViewBasic {
        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        user_id: permission.user_id.clone(),
        role_id: permission.role_id.clone(),

        permissions: permission.permissions as u64,
    };
}

pub fn permission_view_detailed(permission: &CampsitePermission) -> PermissionViewDetailed {
    return PermissionViewDetailed {
        id: permission.id.clone(),
        campsite_id: permission.campsite_id.clone(),

        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        user_id: permission.user_id.clone(),
        role_id: permission.role_id.clone(),

        permissions: permission.permissions as u64,

        created_by: permission.created_by.clone(),
        created_at: serialize_datetime(permission.created_at),
        updated_by: permission.updated_by.clone(),
        updated_at: serialize_datetime(permission.updated_at),
    };
}
