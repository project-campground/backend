use appview_schema::models::appview::{Actor, Bonfire, Campsite, CampsiteMember, CampsitePermission, CampsiteRole, Profile};
use campground_lexicon::gg::campground::{campsite::{BonfireViewBasic, BonfireViewDetailed, CampsiteMemberViewBasic, CampsitePermissionView, CampsiteRoleViewBasic, CampsiteViewBasic, CampsiteViewDetailed}, tent::{TentCategoryView, TentViewBasic}};
use uuid::Uuid;

use crate::helpers::{util::serialize_datetime, views::profile_view_basic_from_db};

pub fn campsite_view_basic(campsite: &Campsite) -> CampsiteViewBasic {
    return CampsiteViewBasic {
        id: campsite.id.clone(),
        name: campsite.name.clone(),
        vanity_url: campsite.vanity_url.clone(),
        description: campsite.description.clone(),
        avatar_uri: campsite.avatar_uri.clone(),
        banner_uri: campsite.banner_uri.clone(),
        tags: campsite.tags
            .iter()
            .filter_map(|x| x.clone())
            .collect(),
        member_count: campsite.member_dids.len(),
        owner: campsite.owner.clone(),
        created_by: campsite.created_by.clone(),
        created_at: serialize_datetime(campsite.created_at),
        updated_by: campsite.updated_by.clone(),
        updated_at: serialize_datetime(campsite.updated_at)
    };
}

pub fn campsite_view_detailed(campsite: &Campsite, bonfires: Vec<BonfireViewBasic>, roles: Vec<CampsiteRoleViewBasic>) -> CampsiteViewDetailed {
    return CampsiteViewDetailed {
        id: campsite.id.clone(),
        name: campsite.name.clone(),
        vanity_url: campsite.vanity_url.clone(),
        description: campsite.description.clone(),
        avatar_uri: campsite.avatar_uri.clone(),
        banner_uri: campsite.banner_uri.clone(),
        tags: campsite.tags
            .iter()
            .filter_map(|x| x.clone())
            .collect(),
        member_count: campsite.member_dids.len(),
        owner: campsite.owner.clone(),
        created_by: campsite.created_by.clone(),
        created_at: serialize_datetime(campsite.created_at),
        updated_by: campsite.updated_by.clone(),
        updated_at: serialize_datetime(campsite.updated_at),
        bonfires: bonfires,
        roles: roles,
    };
}

pub fn campsite_member_view_basic(campsite_member: &CampsiteMember, profile: &Profile, actor: &Actor) -> CampsiteMemberViewBasic {
    return CampsiteMemberViewBasic {
        user: profile_view_basic_from_db(actor, profile),
        user_id: campsite_member.user_id.clone(),
        campsite_id: campsite_member.campsite_id.clone(),
        joined_at: serialize_datetime(campsite_member.joined_at),
        nickname: campsite_member.nickname.clone(),
        roles: campsite_member.roles
            .iter()
            .filter_map(|&x| x)
            .collect::<Vec<Uuid>>()
    };
}

pub fn campsite_role_view_basic(role: &CampsiteRole) -> CampsiteRoleViewBasic {
    return CampsiteRoleViewBasic {
        id: role.id.clone(),
        campsite_id: role.campsite_id.clone(),
        name: role.name.clone(),
        display_separately: role.display_separately.clone(),
        mentionable: role.mentionable.clone(),
        color: role.color,
        color_secondary: role.color_secondary,
        campsite_permissions: role.campsite_permissions,
        tent_permissions: role.tent_permissions,
        priority: role.priority,
        created_by: role.created_by.clone(),
        created_at: serialize_datetime(role.created_at),
        updated_by: role.updated_by.clone(),
        updated_at: serialize_datetime(role.updated_at),
    };
}

pub fn campsite_permission_view(permission: CampsitePermission) -> CampsitePermissionView {
    return CampsitePermissionView {
        id: permission.id.clone(),
        campsite_id: permission.campsite_id.clone(),

        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        user_id: permission.user_id.clone(),
        role_id: permission.role_id.clone(),

        allowed_campsite_permissions: permission.allowed_campsite_permissions,
        denied_campsite_permissions: permission.denied_campsite_permissions,
        allowed_tent_permissions: permission.allowed_tent_permissions,
        denied_tent_permissions: permission.denied_tent_permissions,

        created_by: permission.created_by.clone(),
        created_at: serialize_datetime(permission.created_at),
        updated_by: permission.updated_by.clone(),
        updated_at: serialize_datetime(permission.updated_at),
    };
}

// pub fn campsite_member_view_detailed(campsite_member: &CampsiteMember) -> CampsiteMemberViewDetailed {
//     return CampsiteMemberViewDetailed {
//         user_id: campsite_member.user_id.clone(),
//         campsite_id: campsite_member.campsite_id.clone(),
//         joined_at: serialize_datetime(campsite_member.joined_at),
//         used_invite_id: campsite_member.used_invite_id.clone(),
//         nickname: campsite_member.nickname.clone(),
//     };
// }

pub fn bonfire_view_basic(bonfire: &Bonfire) -> BonfireViewBasic {
    return BonfireViewBasic {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite_id.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar_uri.clone(),
        banner_uri: bonfire.banner_uri.clone(),
        priority: bonfire.priority,
        created_by: bonfire.created_by.clone(),
        created_at: serialize_datetime(bonfire.created_at),
        updated_by: bonfire.updated_by.clone(),
        updated_at: serialize_datetime(bonfire.updated_at)
    };
}
pub fn bonfire_view_detailed(bonfire: &Bonfire, tents: Vec<TentViewBasic>, categories: Vec<TentCategoryView>) -> BonfireViewDetailed {
    return BonfireViewDetailed {
        id: bonfire.id.clone(),
        campsite_id: bonfire.campsite_id.clone(),
        name: bonfire.name.clone(),
        description: bonfire.description.clone(),
        avatar_uri: bonfire.avatar_uri.clone(),
        banner_uri: bonfire.banner_uri.clone(),
        priority: bonfire.priority,
        created_by: bonfire.created_by.clone(),
        created_at: serialize_datetime(bonfire.created_at),
        updated_by: bonfire.updated_by.clone(),
        updated_at: serialize_datetime(bonfire.updated_at),
        tents,
        categories
    };
}
