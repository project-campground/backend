#![
    allow(dead_code)
]
use appview_schema::{models::appview::{Campsite, CampsiteMember, CampsitePermission, CampsiteRole}, schema::appview::campsite_permission};
use diesel::{BoolExpressionMethods, ExpressionMethods, PgSortExpressionMethods, QueryDsl, RunQueryDsl, dsl::sql, sql_types::Bool};
use uuid::Uuid;

use crate::{database::{campsites::get_specific_roles, establish_connection}, helpers::api::handle_select_first_error, xrpc::error::XRPCError};

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
    
    let (campsite_perms_from_role, tent_perms_from_role) = aggregate_role_permissions(roles);
    
    Ok((tent_perms & tent_perms_from_role == tent_perms) && (campsite_perms & campsite_perms_from_role == campsite_perms))
}
pub async fn has_role_perms_or_owner(campsite: &Campsite, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    if campsite.owner == member.user_id { Ok(true) } else { has_role_permissions(member, campsite_perms, tent_perms).await }
}
pub async fn has_role_perms_from_roles_or_owner(campsite: &Campsite, member: &CampsiteMember, roles: &Vec<CampsiteRole>, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    if campsite.owner == member.user_id {
        Ok(true)
    } else {
        let (campsite_perms_from_role, tent_perms_from_role) = aggregate_member_permissions(member, roles);
        
        Ok((tent_perms & tent_perms_from_role == tent_perms) && (campsite_perms & campsite_perms_from_role == campsite_perms))
    }
}
pub async fn has_tent_perms_or_owner(campsite: &Campsite, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    if campsite.owner == member.user_id { Ok(true) } else { has_full_tent_perms(&campsite.id, &bonfire_id, category_id, tent_id, member, campsite_perms, tent_perms).await }
}
pub async fn has_full_tent_perms(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    let member_roles: &Vec<Uuid> = &member.roles.iter().filter_map(|&x| x).collect();
    let (_perms, given_camp_perms, given_tent_perms) = get_tent_permissions(campsite_id, bonfire_id, category_id, tent_id, member.user_id.as_str(), member_roles).await?;

    // Has all the needed perms
    if (given_camp_perms.0 & campsite_perms) == campsite_perms && (given_tent_perms.0 & tent_perms) == tent_perms {
        return Ok(true);
        // One of the perms is denied
    } else if (given_camp_perms.1 & campsite_perms) != 0 || (given_tent_perms.1 & tent_perms) != 0 {
        return Ok(false);
    }
    
    let roles = &get_specific_roles(member_roles)?;
    let (role_camp_perms, role_tent_perms) = aggregate_member_permissions(member, roles);
    
    Ok((role_camp_perms & campsite_perms) == campsite_perms && (role_tent_perms & tent_perms) == tent_perms)
}
pub async fn has_full_tent_perms_from_roles(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, roles: &Vec<CampsiteRole>, member: &CampsiteMember, campsite_perms: i64, tent_perms: i64) -> Result<bool, XRPCError> {
    let member_roles: &Vec<Uuid> = &member.roles.iter().filter_map(|&x| x).collect();
    let (_perms, given_camp_perms, given_tent_perms) = get_tent_permissions(campsite_id, bonfire_id, category_id, tent_id, &member.user_id, member_roles).await?;
    
    // Has all the needed perms
    if (given_camp_perms.0 & campsite_perms) == campsite_perms && (given_tent_perms.0 & tent_perms) == tent_perms {
        return Ok(true);
        // One of the perms is denied
    } else if (given_camp_perms.1 & campsite_perms) != 0 || (given_tent_perms.1 & tent_perms) != 0 {
        return Ok(false);
    }

    let (role_camp_perms, role_tent_perms) = aggregate_member_permissions(member, roles);
    
    Ok((role_camp_perms & campsite_perms) == campsite_perms && (role_tent_perms & tent_perms) == tent_perms)
}
pub async fn get_tent_permissions(campsite_id: &str, bonfire_id: &str, category_id: Option<Uuid>, tent_id: Option<Uuid>, actor: &str, role_ids: &Vec<Uuid>) -> Result<(Vec<CampsitePermission>, (i64, i64), (i64, i64)), XRPCError> {
    let campsite_perms = fetch_tent_permissions(campsite_id, bonfire_id, category_id, tent_id, actor, role_ids).await?;
    
    let (campsite_permissions, tent_permissions) = aggregate_permissions(&campsite_perms);

    Ok((campsite_perms, campsite_permissions, tent_permissions))
}
pub fn aggregate_permissions(perms: &Vec<CampsitePermission>) -> ((i64, i64), (i64, i64)) {
    // (campsite(allowed, disallowed), tent(allowed, disallowed))
    let mut aggregated_perms = ((0i64, 0i64), (0i64, 0i64));
    perms
        .iter()
        .for_each(|perm| {
            flip_perms(&mut aggregated_perms.0, perm.allowed_campsite_permissions, perm.denied_campsite_permissions);
            flip_perms(&mut aggregated_perms.1, perm.allowed_tent_permissions, perm.denied_tent_permissions);
        });

    (aggregated_perms.0, aggregated_perms.1)
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
                            sql::<Bool>(
                                tent_id
                                    .map_or(
                                        "FALSE".to_string(),
                                        |x| format!("\"campsite_permission\".\"tentid\" = '{}'", x)
                                    )
                                    .as_str()
                            )
                        )
                        .or(
                            sql::<Bool>(
                                category_id
                                    .map_or(
                                        "FALSE".to_string(),
                                        |x| format!("\"campsite_permission\".\"categoryid\" = '{}'", x)
                                    )
                                    .as_str()
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
pub fn aggregate_member_permissions(member: &CampsiteMember, roles: &Vec<CampsiteRole>) -> (i64, i64) {
    let member_roles = &roles
        .iter()
        .filter(|role|
            member.roles.contains(&Some(role.id))
        )
        .collect::<Vec<&CampsiteRole>>();
    
    let campsite_perms = member_roles
        .iter()
        .fold(0i64, |camp_perm, role|
            camp_perm | role.campsite_permissions
        );
    let tent_perms = member_roles
        .iter()
        .fold(0i64, |tent_perm, role|
            tent_perm | role.tent_permissions
        );
    
    (campsite_perms, tent_perms)
}
pub fn aggregate_role_permissions(roles: &Vec<CampsiteRole>) -> (i64, i64) {
    let campsite_perms = roles
        .iter()
        .fold(0i64, |camp_perm, role|
            camp_perm | role.campsite_permissions
        );
    let tent_perms = roles
        .iter()
        .fold(0i64, |tent_perm, role|
            tent_perm | role.tent_permissions
        );
    
    (campsite_perms, tent_perms)
}
fn flip_perms(tuple: &mut (i64, i64), allowed: i64, disallowed: i64) {
    tuple.0 |= allowed;
    tuple.1 |= disallowed;
}

