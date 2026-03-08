#![
    allow(dead_code)
]

use appview_schema::{models::appview::{Campsite, CampsiteMember, CampsitePermission, CampsiteRole}, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::permission::{PermissionsDictionary, PermissionsStateDictionary};
use diesel::{BoolExpressionMethods, ExpressionMethods, PgSortExpressionMethods, QueryDsl, RunQueryDsl, sql_types::Bool};
use uuid::Uuid;

use crate::{database::{campsites::get_specific_roles, establish_connection}, helpers::api::{handle_all_db_errors, handle_select_first_error}, util::iter::AggregatePermissions, xrpc::error::XRPCError};

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
    let (_perms, given_perms) = get_tent_permissions(campsite_id, bonfire_id, category_id, tent_id, member.user_id.as_str(), member_roles).await?;

    // Has all the needed perms
    if (given_perms.allowed.campsite & campsite_perms) == campsite_perms && (given_perms.allowed.tent & tent_perms) == tent_perms {
        return Ok(true);
        // One of the perms is denied
    } else if (given_perms.denied.campsite & campsite_perms) != 0 || (given_perms.denied.tent & tent_perms) != 0 {
        return Ok(false);
    }
    
    let roles = &get_specific_roles(member_roles)?;
    let role_perms = aggregate_member_permissions(member, roles);
    
    Ok((role_perms.campsite & campsite_perms) == campsite_perms && (role_perms.tent & tent_perms) == tent_perms)
}
pub async fn has_full_tent_perms_from_roles(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, roles: &Vec<CampsiteRole>, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    let member_roles: &Vec<Uuid> = &member.roles.iter().filter_map(|&x| x).collect();
    let (_perms, given_perms) = get_tent_permissions(campsite_id, bonfire_id, category_id, tent_id, &member.user_id, member_roles).await?;
    
    // Has all the needed perms
    if (given_perms.allowed.campsite & campsite_perms) == campsite_perms && (given_perms.allowed.tent & tent_perms) == tent_perms {
        return Ok(true);
        // One of the perms is denied
    } else if (given_perms.denied.campsite & campsite_perms) != 0 || (given_perms.denied.tent & tent_perms) != 0 {
        return Ok(false);
    }

    let role_perms = aggregate_member_permissions(member, roles);
    
    Ok((role_perms.campsite & campsite_perms) == campsite_perms && (role_perms.tent & tent_perms) == tent_perms)
}
pub async fn get_tent_permissions(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, actor: &str, role_ids: &Vec<Uuid>) -> Result<(Vec<CampsitePermission>, PermissionsStateDictionary), XRPCError> {
    let campsite_perms = fetch_tent_permissions(campsite_id, bonfire_id, category_id, tent_id, actor, role_ids).await?;
    
    let perms = campsite_perms
        .iter()
        .aggregate_permissions();

    Ok((campsite_perms, perms))
}
pub fn fetch_all_campsite_permissions(campsite_id: &str, actor: &str) -> Result<Vec<CampsitePermission>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    
    campsite_permission::table
        .filter(
            campsite_permission::campsiteid
                .eq(
                    campsite_id
                )
                .and(
                    campsite_permission::userid
                        .is_null()
                        .or(
                            campsite_permission::userid
                                .eq(actor)
                        )
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)
}
pub async fn fetch_tent_permissions(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, actor: &str, role_ids: &Vec<Uuid>) -> Result<Vec<CampsitePermission>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    campsite_permission::table
        .filter(
            campsite_permission::campsiteid
                .eq(
                    campsite_id
                )
                // Of who
                .and(
                    campsite_permission::roleid
                        .eq_any(role_ids)
                        .or(
                            campsite_permission::userid
                                .eq(actor)
                        )
                )
                // Where
                .and(
                    campsite_permission::bonfireid
                        .eq(
                            bonfire_id
                        )
                        .or(
                            campsite_permission::tentid
                                .is_not_null()
                                .and(
                                    campsite_permission::tentid
                                        .eq(tent_id)
                                )
                        )
                        .or(
                            campsite_permission::categoryid
                                .is_not_null()
                                .and(
                                    campsite_permission::categoryid
                                        .eq(category_id)
                                )
                            
                        )
                )
        )
        // Force bonfire, then category, then tent perms to be applied
        .order((
            campsite_permission::bonfireid.asc().nulls_last(),
            campsite_permission::categoryid.asc().nulls_last(),
            campsite_permission::tentid.asc().nulls_last(),
        ))
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)
}
pub async fn fetch_only_specific_permissions(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, actor: &str, role_ids: &Vec<Uuid>) -> Result<Vec<CampsitePermission>, XRPCError> {
    let mut conn = establish_connection().unwrap();
    campsite_permission::table
        .filter(
            campsite_permission::campsiteid
                .eq(
                    campsite_id
                )
                // Of who
                .and(
                    campsite_permission::roleid
                        .eq_any(role_ids)
                        .or(
                            campsite_permission::userid
                                .eq(actor)
                        )
                )
                // Where
                .and(
                    // Either bonfire
                    campsite_permission::bonfireid
                        .eq(
                            bonfire_id
                        )
                        .and(
                            campsite_permission::tentid
                                .is_null()
                                .and::<bool, Bool>(
                                    tent_id.is_none()
                                )
                        )
                        .and(
                            campsite_permission::categoryid
                                .is_null()
                                .and::<bool, Bool>(
                                    category_id.is_none()
                                )
                        )
                        // Or tent if specified
                        .or(
                            campsite_permission::tentid
                                .is_not_null()
                                .and(
                                    campsite_permission::tentid
                                        .eq(tent_id)
                                )
                        )
                        // Or category if specified
                        .or(
                            campsite_permission::categoryid
                                .is_not_null()
                                .and(
                                    campsite_permission::categoryid
                                        .eq(category_id)
                                )
                            
                        )
                )
        )
        // Force bonfire, then category, then tent perms to be applied
        .order((
            campsite_permission::bonfireid.asc().nulls_last(),
            campsite_permission::categoryid.asc().nulls_last(),
            campsite_permission::tentid.asc().nulls_last(),
        ))
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_select_first_error)
}
pub fn aggregate_member_permissions(member: &CampsiteMember, roles: &Vec<CampsiteRole>) -> PermissionsDictionary {
    roles
        .iter()
        .filter(|role|
            member.roles.contains(&Some(role.id))
        )
        .aggregate_permissions()
}
