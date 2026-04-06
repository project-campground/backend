use appview_schema::models::appview::CampsitePermission;
use campground_lexicon::gg::campground::
    permission::{
        PermissionViewBasic, PermissionViewDetailed, PermissionsDictionary,
        PermissionsStateDictionary,
    }
;

use crate::views::util::serialize_datetime;

pub fn permission_view_basic(permission: &CampsitePermission) -> PermissionViewBasic {
    return PermissionViewBasic {
        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        user_id: permission.user_id.clone(),
        role_id: permission.role_id.clone(),

        permissions: PermissionsStateDictionary {
            allowed: PermissionsDictionary {
                general: permission.allowed_general_permissions as u64,
                content: permission.allowed_content_permissions as u64,
            },
            denied: PermissionsDictionary {
                general: permission.denied_general_permissions as u64,
                content: permission.denied_content_permissions as u64,
            },
        },
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

        permissions: PermissionsStateDictionary {
            allowed: PermissionsDictionary {
                general: permission.allowed_general_permissions as u64,
                content: permission.allowed_content_permissions as u64,
            },
            denied: PermissionsDictionary {
                general: permission.denied_general_permissions as u64,
                content: permission.denied_content_permissions as u64,
            },
        },

        created_by: permission.created_by.clone(),
        created_at: serialize_datetime(permission.created_at),
        updated_by: permission.updated_by.clone(),
        updated_at: serialize_datetime(permission.updated_at),
    };
}
