#![
    allow(dead_code)
]

use appview_schema::models::appview::{Campsite, CampsiteMember, CampsitePermission, CampsiteRole};
use campground_lexicon::gg::campground::permission::PermissionsDictionary;
use uuid::Uuid;

use crate::{database::campsites::get_specific_roles, helpers::permissions::{fetch::fetch_tent_permissions, state::PermissionState}, util::iter::AggregatePermissions, xrpc::error::XRPCError};

pub struct CampsitePermissionConsts { }
impl CampsitePermissionConsts {
    pub const MANAGE_CAMPSITE: i64 = 0b1;
    pub const MANAGE_BONFIRES: i64 = 0b10;
    pub const MANAGE_TENTS: i64 = 0b100;
    pub const MANAGE_ROLES: i64 = 0b1000;
    pub const GIVE_ROLES: i64 = 0b10000;
    pub const MUTE_MEMBERS: i64 = 0b100000;
    pub const KICK_MEMBERS: i64 = 0b1000000;
    pub const BAN_MEMBERS: i64 = 0b10000000;
    pub const MANAGE_SELF_IDENTITY: i64 = 0b100000000;
    pub const MANAGE_OTHERS_IDENTITY: i64 = 0b1000000000;
    pub const CREATE_INVITES: i64 = 0b10000000000;
    pub const MANAGE_INVITES: i64 = 0b100000000000;
    pub const MAX: i64 = 0b111111111111;
}
pub struct TentPermissionConsts { }
impl TentPermissionConsts {
    pub const VIEW_CONTENT: i64 = 0b1;
    pub const CREATE_CONTENT: i64 = 0b10;
    pub const PIN_CONTENT: i64 = 0b100;
    pub const MANAGE_CONTENT: i64 = 0b1000;
    pub const MENTION_EVERYONE: i64 = 0b10000;
    pub const CREATE_PRIVATE_CONTENT: i64 = 0b100000;
    pub const MAX: i64 = 0b111111;
}

pub async fn has_role_permissions(member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    let member_role_ids: &Vec<Uuid> = &member.roles.iter().filter_map(|&x| x).collect::<Vec<Uuid>>();
    let roles = &get_specific_roles(member_role_ids)?;
    
    let perms = roles
        .iter()
        .aggregate_permissions();
    
    Ok((tent_perms & perms.tent == tent_perms) && (campsite_perms & perms.campsite == campsite_perms))
}
pub async fn has_role_perms_or_owner(campsite: &Campsite, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    if campsite.owner == member.user_id { Ok(true) } else { has_role_permissions(member, campsite_perms, tent_perms).await }
}
pub async fn has_role_perms_from_roles_or_owner(campsite: &Campsite, member: &CampsiteMember, roles: &Vec<CampsiteRole>, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    if campsite.owner == member.user_id {
        Ok(true)
    } else {
        let perms_from_role = aggregate_member_permissions(member, roles);
        
        Ok((tent_perms & perms_from_role.tent == tent_perms) && (campsite_perms & perms_from_role.campsite == campsite_perms))
    }
}
pub async fn has_tent_perms_or_owner(campsite: &Campsite, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    if campsite.owner == member.user_id { Ok(true) } else { has_full_tent_perms(&campsite.id, &bonfire_id, category_id, tent_id, member, campsite_perms, tent_perms).await }
}
pub async fn has_full_tent_perms(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    let member_roles: &Vec<Uuid> = &member.roles.iter().filter_map(|&x| x).collect();
    let (_perms, campsite_perm_state, tent_perm_state) = aggregate_leveled_permission(campsite_id, bonfire_id, category_id, tent_id, member.user_id.as_str(), member_roles, campsite_perms, tent_perms).await?;

    // Has all the needed perms
    if campsite_perm_state == tent_perm_state && tent_perm_state == PermissionState::Allowed {
        return Ok(true);
        // One of the perms is denied
    } else if campsite_perm_state == PermissionState::Denied || tent_perm_state == PermissionState::Denied {
        return Ok(false);
    }

    let roles = &get_specific_roles(member_roles)?;
    let role_perms = aggregate_member_permissions(member, roles);

    Ok(
        (campsite_perm_state == PermissionState::Allowed || (role_perms.campsite & campsite_perms) == campsite_perms) &&
        (tent_perm_state == PermissionState::Allowed || (role_perms.tent & tent_perms) == tent_perms)
    )
}
pub async fn has_full_tent_perms_from_roles(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, roles: &Vec<CampsiteRole>, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    let member_roles: &Vec<Uuid> = &member.roles.iter().filter_map(|&x| x).collect();
    let (_perms, campsite_perm_state, tent_perm_state) = aggregate_leveled_permission(campsite_id, bonfire_id, category_id, tent_id, &member.user_id, member_roles, campsite_perms, tent_perms).await?;

    // Has all the needed perms
    if campsite_perm_state == tent_perm_state && tent_perm_state == PermissionState::Allowed {
        return Ok(true);
        // One of the perms is denied
    } else if campsite_perm_state == PermissionState::Denied || tent_perm_state == PermissionState::Denied {
        return Ok(false);
    }

    let role_perms = aggregate_member_permissions(member, roles);

    Ok(
        (campsite_perm_state == PermissionState::Allowed || (role_perms.campsite & campsite_perms) == campsite_perms) &&
        (tent_perm_state == PermissionState::Allowed || (role_perms.tent & tent_perms) == tent_perms)
    )
}
pub async fn aggregate_leveled_permission(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, actor: &str, role_ids: &Vec<Uuid>, campsite_perms: i64, tent_perms: i64) -> Result<(Vec<CampsitePermission>, PermissionState, PermissionState), XRPCError> {
    let perms_vec = fetch_tent_permissions(campsite_id, bonfire_id, category_id, tent_id, actor, role_ids)
        .await?;

    let perms = perms_vec.iter();

    let tent_level_perms = perms
        .clone()
        .filter(|x| x.tent_id.is_some())
        .aggregate_permissions();
    let category_level_perms = perms
        .clone()
        .filter(|x| x.category_id.is_some())
        .aggregate_permissions();
    let bonfire_level_perms = perms
        .filter(|x| x.tent_id.is_none() && x.category_id.is_none())
        .aggregate_permissions();

    let tent_perms_state = PermissionState::from_tent_three_level(&bonfire_level_perms, &category_level_perms, &tent_level_perms, tent_perms);
    let campsite_perms_state = PermissionState::from_campsite_three_level(&bonfire_level_perms, &category_level_perms, &tent_level_perms, campsite_perms);

    Ok((perms_vec, campsite_perms_state, tent_perms_state))
}
pub fn aggregate_member_permissions(member: &CampsiteMember, roles: &Vec<CampsiteRole>) -> PermissionsDictionary {
    roles
        .iter()
        .filter(|role|
            member.roles.contains(&Some(role.id))
        )
        .aggregate_permissions()
}

pub mod state;
pub mod fetch;