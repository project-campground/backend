use std::{collections::HashMap, hash::Hash};

use appview_schema::{models::appview::{CampsiteMember, CampsitePermission, CampsiteRole}, schema::appview::campsite_permission};
use campground_lexicon::gg::campground::permission::{PermissionsDictionary, PermissionsStateDictionary};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use multimap::MultiMap;
use uuid::Uuid;

use crate::{database::{campsites::get_roles_from_db, establish_connection}, helpers::{api::handle_all_db_errors, ws::CampsiteMemberPermissions}, util::iter::{AggregatePermissions, aggregate_permissions_double_ref}, xrpc::error::XRPCError};

#[derive(PartialEq, Eq)]
pub enum PermissionState {
    Allowed,
    Inherit,
    Denied,
}

impl PermissionState {
    pub fn map_inherit<F: FnOnce() -> PermissionState>(self, map: F) -> PermissionState {
        match self {
            PermissionState::Inherit => map(),
            _ => self,
        }
    }
    pub fn from_role_tent(permissions: &PermissionsDictionary, flag: i64) -> PermissionState {
        if permissions.tent & flag == flag { PermissionState::Allowed } else { PermissionState::Denied }
    }
    #[allow(dead_code)]
    pub fn from_role_campsite(permissions: &PermissionsDictionary, flag: i64) -> PermissionState {
        if permissions.campsite & flag == flag { PermissionState::Allowed } else { PermissionState::Denied }
    }
    pub fn from_tent_optional(permission: &Option<&PermissionsStateDictionary>, flag: i64) -> PermissionState {
        permission.map_or(PermissionState::Inherit, |x| PermissionState::from_tent(x, flag))
    }
    pub fn from_tent(permission: &PermissionsStateDictionary, flag: i64) -> PermissionState {
        if permission.allowed.tent & flag == flag {
            PermissionState::Allowed
        } else if permission.denied.tent & flag == flag {
            PermissionState::Denied
        } else {
            PermissionState::Inherit
        }
    }
    #[allow(dead_code)]
    pub fn from_campsite_optional(permission: &Option<&PermissionsStateDictionary>, flag: i64) -> PermissionState {
        permission.map_or(PermissionState::Inherit, |x| PermissionState::from_campsite(x, flag))
    }
    pub fn from_campsite(permission: &PermissionsStateDictionary, flag: i64) -> PermissionState {
        if permission.allowed.campsite & flag == flag {
            PermissionState::Allowed
        } else if permission.denied.campsite & flag == flag {
            PermissionState::Denied
        } else {
            PermissionState::Inherit
        }
    }
    pub fn is_allowed(self) -> bool {
        self == PermissionState::Allowed
    }
}

pub async fn update_ws_permissions<'a>(campsite_id: &str, member: &CampsiteMember, current: &CampsiteMemberPermissions, no_role_permissions: bool, updated_bonfires: Vec<String>, updated_categories: Vec<Uuid>, updated_tents: Vec<Uuid>) -> Result<CampsiteMemberPermissions, XRPCError> {    
    let mut conn = establish_connection().unwrap();

    let permissions = campsite_permission::table
        .filter(
            campsite_permission::bonfireid
                .eq_any(updated_bonfires)
                .and(
                    campsite_permission::tentid
                        .is_null()
                        .and(
                            campsite_permission::categoryid
                            .is_null()
                        )
                )
                .or(
                    campsite_permission::tentid
                        .eq_any(updated_tents)
                )
                .or(
                    campsite_permission::categoryid
                        .eq_any(updated_categories)
                )
        )
        .load::<CampsitePermission>(&mut conn)
        .map_err(handle_all_db_errors)?;

    let bonfires: HashMap<String, PermissionsStateDictionary> =
        aggregate_ws_item_permissions(
            permissions
                .iter()
                .filter(|x| x.tent_id.is_none() && x.category_id.is_none())
            , |perm| perm.bonfire_id.clone()
        );
    let categories: HashMap<Uuid, PermissionsStateDictionary> =
        aggregate_ws_item_permissions(
            permissions
                .iter()
                .filter(|x| x.category_id.is_some())
            , |perm| perm.category_id.unwrap()
        );
    let tents: HashMap<Uuid, PermissionsStateDictionary> =
        aggregate_ws_item_permissions(
            permissions
                .iter()
                .filter(|x| x.tent_id.is_some())
            , |perm| perm.tent_id.unwrap()
        );

    let merged_bonfires = &mut current.bonfires.clone();
    merged_bonfires.extend(bonfires);

    let merged_categories = &mut current.categories.clone();
    merged_categories.extend(categories);

    let merged_tents = &mut current.tents.clone();
    merged_tents.extend(tents);

    Ok(CampsiteMemberPermissions {
        roles: if no_role_permissions { current.roles.clone() } else {
            get_roles_from_db(&campsite_id)?
                .iter()
                .filter(|x| member.roles.contains(&Some(x.id)))
                .aggregate_permissions()
        },
        bonfires: merged_bonfires.clone(),
        categories: merged_categories.clone(),
        tents: merged_tents.clone(),
    })
}
pub async fn aggregate_ws_permissions<'a>(member: &CampsiteMember, roles: &Vec<CampsiteRole>, permissions: &Vec<CampsitePermission>) -> Result<CampsiteMemberPermissions, XRPCError> {
    let member_role_perms = roles
        .iter()
        .filter(|x| member.roles.contains(&Some(x.id)))
        .aggregate_permissions();

    let bonfires: HashMap<String, PermissionsStateDictionary> =
        aggregate_ws_item_permissions(
            permissions
                .iter()
                .filter(|x| x.tent_id.is_none() && x.category_id.is_none())
            , |perm| perm.bonfire_id.clone()
        );
    let categories: HashMap<Uuid, PermissionsStateDictionary> =
        aggregate_ws_item_permissions(
            permissions
                .iter()
                .filter(|x| x.category_id.is_some())
            , |perm| perm.category_id.unwrap()
        );
    let tents: HashMap<Uuid, PermissionsStateDictionary> =
        aggregate_ws_item_permissions(
            permissions
                .iter()
                .filter(|x| x.tent_id.is_some())
            , |perm| perm.tent_id.unwrap()
        );

    Ok(CampsiteMemberPermissions {
        roles: member_role_perms,
        bonfires,
        categories,
        tents,
    })
}
fn aggregate_ws_item_permissions<'a, K: Clone + Eq + Hash, F: FnMut(&CampsitePermission) -> K, I: Iterator<Item = &'a CampsitePermission>>(permissions: I, mut get_key: F) -> HashMap<K, PermissionsStateDictionary> {
    permissions
        .map(|x| (get_key(x), x))
        .collect::<MultiMap<K, &CampsitePermission>>()
        .iter_all()
        .map(|(key, values)|
            (key.clone(), aggregate_permissions_double_ref(values.iter()))
        )
        .collect::<HashMap<K, PermissionsStateDictionary>>()
}