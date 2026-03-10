use appview_schema::models::appview::{Actor, Bonfire, Campsite, CampsiteBan, CampsiteInvite, CampsiteMember, CampsitePermission, CampsiteRole, Profile};
use campground_lexicon::gg::campground::{actor::ProfileViewBasic, campsite::{BonfireViewBasic, BonfireViewDetailed, CampsitePermissionViewBasic, CampsitePermissionViewDetailed, CampsiteRoleViewBasic, CampsiteViewBasic, CampsiteViewDetailed}, membership::{CampsiteBanView, CampsiteInviteViewCampsite, CampsiteInviteViewGlobal, CampsiteMemberViewAuthor, CampsiteMemberViewBasic, CampsiteMemberViewDetailed}, permission::{PermissionsDictionary, PermissionsStateDictionary}, tent::{TentCategoryView, TentViewBasic}};
use uuid::Uuid;

use crate::helpers::{util::serialize_datetime, views::{profile_view_basic_deleted_profile, profile_view_basic_from_db, profile_view_detailed_from_db}};

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

pub fn campsite_view_detailed(campsite: &Campsite, bonfires: Vec<BonfireViewBasic>, roles: Vec<CampsiteRoleViewBasic>, member: CampsiteMemberViewBasic) -> CampsiteViewDetailed {
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
        member,
    };
}

pub fn campsite_member_view_basic(campsite_member: &CampsiteMember, profile: &Profile, actor: &Actor) -> CampsiteMemberViewBasic {
    return CampsiteMemberViewBasic {
        user: profile_view_basic_from_db(actor, profile),
        nickname: campsite_member.nickname.clone(),
        roles: campsite_member.roles
            .iter()
            .filter_map(|&x| x)
            .collect::<Vec<Uuid>>()
    };
}

pub fn campsite_member_view_detailed(campsite_member: &CampsiteMember, profile: &Profile, actor: &Actor) -> CampsiteMemberViewDetailed {
    return CampsiteMemberViewDetailed {
        campsite_id: campsite_member.campsite_id.clone(),
        used_invite_id: campsite_member.used_invite_id.clone(),
        user: profile_view_detailed_from_db(actor, profile),
        nickname: campsite_member.nickname.clone(),
        roles: campsite_member.roles
            .iter()
            .filter_map(|&x| x)
            .collect::<Vec<Uuid>>(),
        joined_at: serialize_datetime(campsite_member.joined_at),
    };
}

const DEFAULT_NO_ROLES: Vec<Uuid> = vec![];

pub fn campsite_member_view_author(campsite_member: &Option<CampsiteMember>, profile_view: ProfileViewBasic) -> CampsiteMemberViewAuthor {
    return CampsiteMemberViewAuthor {
        is_member: campsite_member.is_some(),
        user: profile_view,
        nickname: campsite_member.clone().map(|member| member.nickname.clone()).flatten(),
        roles: campsite_member.clone().map_or(DEFAULT_NO_ROLES, |member|
            member.roles
                .iter()
                .filter_map(|&x| x)
                .collect::<Vec<Uuid>>()
        )
    };
}

pub fn campsite_invite_view_campsite(campsite_invite: &CampsiteInvite, profile: &Profile, actor: &Actor) -> CampsiteInviteViewCampsite {
    return CampsiteInviteViewCampsite {
        id: campsite_invite.id.clone(),
        expires_at: campsite_invite.expires_at.map(|x| serialize_datetime(x)),
        allowed_amount: campsite_invite.allowed_amount.clone(),
        used: campsite_invite.used,
        created_at: serialize_datetime(campsite_invite.created_at),
        created_by: profile_view_basic_from_db(actor, profile),
    };
}

pub fn campsite_invite_view_global(campsite_invite: &CampsiteInvite, campsite: &Campsite, profile: &Profile, actor: &Actor) -> CampsiteInviteViewGlobal {
    return CampsiteInviteViewGlobal {
        id: campsite_invite.id.clone(),
        campsite: campsite_view_basic(campsite),
        expires_at: campsite_invite.expires_at.map(|x| serialize_datetime(x)),
        allowed_amount: campsite_invite.allowed_amount.clone(),
        used: campsite_invite.used,
        created_at: serialize_datetime(campsite_invite.created_at),
        created_by: profile_view_basic_from_db(actor, profile),
    };
}

pub fn campsite_ban_view(ban: &CampsiteBan, profile: &Option<Profile>, actor: &Actor) -> CampsiteBanView {
    return CampsiteBanView {
        user: match profile {
            Some(profile) => profile_view_basic_from_db(actor, profile),
            None => profile_view_basic_deleted_profile(actor),
        },
        user_id: ban.user_id.clone(),
        campsite_id: ban.campsite_id.clone(),
        reason: ban.reason.clone(),
        created_at: serialize_datetime(ban.created_at),
        created_by: ban.created_by.clone(),
        updated_at: serialize_datetime(ban.updated_at),
        updated_by: ban.updated_by.clone(),
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
        permissions: PermissionsDictionary {
            general: role.general_permissions,
            content: role.content_permissions,
        },
        priority: role.priority,
        created_by: role.created_by.clone(),
        created_at: serialize_datetime(role.created_at),
        updated_by: role.updated_by.clone(),
        updated_at: serialize_datetime(role.updated_at),
        flags: role.flags,
    };
}

pub fn campsite_permission_view_basic(permission: &CampsitePermission) -> CampsitePermissionViewBasic {
    return CampsitePermissionViewBasic {
        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        user_id: permission.user_id.clone(),
        role_id: permission.role_id.clone(),

        permissions: PermissionsStateDictionary {
            allowed: PermissionsDictionary {
                general: permission.allowed_general_permissions,
                content: permission.allowed_content_permissions,
            },
            denied: PermissionsDictionary {
                general: permission.denied_general_permissions,
                content: permission.denied_content_permissions,
            },
        },
    };
}

pub fn campsite_permission_view_detailed(permission: &CampsitePermission) -> CampsitePermissionViewDetailed {
    return CampsitePermissionViewDetailed {
        id: permission.id.clone(),
        campsite_id: permission.campsite_id.clone(),

        bonfire_id: permission.bonfire_id.clone(),
        category_id: permission.category_id.clone(),
        tent_id: permission.tent_id.clone(),

        user_id: permission.user_id.clone(),
        role_id: permission.role_id.clone(),

        permissions: PermissionsStateDictionary {
            allowed: PermissionsDictionary {
                general: permission.allowed_general_permissions,
                content: permission.allowed_content_permissions,
            },
            denied: PermissionsDictionary {
                general: permission.denied_general_permissions,
                content: permission.denied_content_permissions,
            },
        },

        created_by: permission.created_by.clone(),
        created_at: serialize_datetime(permission.created_at),
        updated_by: permission.updated_by.clone(),
        updated_at: serialize_datetime(permission.updated_at),
    };
}

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
