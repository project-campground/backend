use appview_schema::{
    models::appview::{Campsite, CampsiteMember, CampsiteRole},
    schema::appview::campsite_role,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    database::establish_connection,
    helpers::{
        api::handle_select_first_error,
        permissions::{fetch::fetch_leveled_permissions, state::PermissionState},
    },
    util::iter::AggregatePermissions,
    xrpc::error::XRPCError,
};

#[derive(Clone)]
pub struct PermissionCheck {
    pub actor: String,
    pub roles: Vec<Uuid>,
}

impl PermissionCheck {
    pub fn member(member: &CampsiteMember) -> PermissionCheck {
        PermissionCheck {
            actor: member.user_id.clone(),
            roles: member
                .roles
                .iter()
                .filter_map(|x| x.clone())
                .collect::<Vec<Uuid>>(),
        }
    }

    pub fn is_owner(self, campsite: &Campsite) -> PermissionCheckWithOwner {
        let is_owner = campsite.owner == self.actor;

        PermissionCheckWithOwner {
            basic: self,
            is_owner,
        }
    }

    pub fn never_owner(self) -> PermissionCheckWithOwner {
        PermissionCheckWithOwner {
            basic: self,
            is_owner: false,
        }
    }
}
pub struct PermissionCheckWithOwner {
    pub basic: PermissionCheck,
    pub is_owner: bool,
}

impl PermissionCheckWithOwner {
    pub fn skip_if_owner<T: FnOnce(PermissionCheckWithOwner) -> Result<bool, XRPCError>>(
        self,
        otherwise: T,
    ) -> Result<bool, XRPCError> {
        if self.is_owner {
            Ok(true)
        } else {
            otherwise(self)
        }
    }

    pub fn fetch_roles(self) -> PermissionCheckWithRolesLazy {
        PermissionCheckWithRolesLazy { with_owner: self }
    }
    pub fn with_roles(self, roles: &Vec<CampsiteRole>) -> PermissionCheckWithRolesEager {
        let role_permissions = roles
            .iter()
            .filter(|x| self.basic.roles.contains(&x.id))
            .fold(0u32, |initial, value| initial | (value.permissions as u32));

        PermissionCheckWithRolesEager { role_permissions }
    }
}

pub trait PermissionCheckTestable {
    fn get_common_permissions(&self, flags: u32) -> Result<u32, XRPCError>;
    fn has_permissions(&self, flags: u32) -> Result<bool, XRPCError> {
        Ok(self.get_common_permissions(flags)? == flags)
    }
    fn has_any_permissions(&self, flags: u32) -> Result<bool, XRPCError> {
        Ok(self.get_common_permissions(flags)? != 0)
    }
}

pub trait PermissionCheckWithRoles: PermissionCheckTestable {
    fn get_roles(&self) -> &Vec<Uuid>;
    fn get_actor(&self) -> &str;
}
pub struct PermissionCheckWithRolesEager {
    pub role_permissions: u32,
}
pub struct PermissionCheckWithRolesLazy {
    pub with_owner: PermissionCheckWithOwner,
}

pub fn scoped_permission<T: PermissionCheckWithRoles>(
    perm: T,
    campsite: &str,
    bonfire: &str,
    category: Option<&Uuid>,
    tent: Option<&Uuid>,
) -> PermissionCheckScoped<T> {
    PermissionCheckScoped {
        with_roles: perm,
        campsite: campsite.to_string(),
        bonfire: bonfire.to_string(),
        category: category.cloned(),
        tent: tent.cloned(),
    }
}
impl PermissionCheckTestable for PermissionCheckWithRolesEager {
    fn get_common_permissions(&self, flags: u32) -> Result<u32, XRPCError> {
        Ok(self.role_permissions & flags)
    }
}
impl PermissionCheckTestable for PermissionCheckWithRolesLazy {
    fn get_common_permissions(&self, flags: u32) -> Result<u32, XRPCError> {
        let mut conn = establish_connection().unwrap();

        let role_permissions = campsite_role::table
            .filter(campsite_role::id.eq_any(&self.with_owner.basic.roles))
            .select(campsite_role::permissions)
            .load::<i32>(&mut conn)
            .map_err(handle_select_first_error)?
            .iter()
            .fold(0u32, |initial, &value| initial | (value as u32));

        Ok(flags & role_permissions)
    }
}
impl PermissionCheckWithRoles for PermissionCheckWithRolesLazy {
    fn get_roles(&self) -> &Vec<Uuid> {
        &self.with_owner.basic.roles
    }
    fn get_actor(&self) -> &str {
        &self.with_owner.basic.actor
    }
}
pub struct PermissionCheckScoped<T: PermissionCheckWithRoles> {
    pub with_roles: T,
    pub campsite: String,
    pub bonfire: String,
    pub category: Option<Uuid>,
    pub tent: Option<Uuid>,
}
impl<T: PermissionCheckWithRoles> PermissionCheckScoped<T> {
    fn get_campsite_permissions(&self, flags: u32) -> Result<PermissionState, XRPCError> {
        let perms_vec = fetch_leveled_permissions(
            &self.campsite,
            &self.bonfire,
            self.category,
            self.tent,
            self.with_roles.get_actor(),
            self.with_roles.get_roles(),
        )?;

        let perms = perms_vec.iter();

        let tent_level_perms = perms
            .clone()
            .filter(|x| x.1.is_some())
            .aggregate_permissions();
        let category_level_perms = perms
            .clone()
            .filter(|x| x.0.is_some())
            .aggregate_permissions();
        let bonfire_level_perms = perms
            .filter(|x| x.0.is_none() && x.1.is_none())
            .aggregate_permissions();

        Ok(PermissionState::from_three_level(
            &bonfire_level_perms,
            &category_level_perms,
            &tent_level_perms,
            flags,
        ))
    }
}
impl<T: PermissionCheckWithRoles> PermissionCheckTestable for PermissionCheckScoped<T> {
    fn get_common_permissions(&self, flags: u32) -> Result<u32, XRPCError> {
        let state = self.get_campsite_permissions(flags)?;

        match state {
            PermissionState::Allowed => Ok(flags),
            PermissionState::Inherit(values) => Ok(values),
            PermissionState::Denied => Ok(0),
        }
    }
    fn has_permissions(&self, flags: u32) -> Result<bool, XRPCError> {
        let state = self.get_campsite_permissions(flags)?;

        match state {
            PermissionState::Allowed => Ok(true),
            PermissionState::Inherit(leftovers) => self.with_roles.has_permissions(leftovers),
            PermissionState::Denied => Ok(false),
        }
    }
    fn has_any_permissions(&self, flags: u32) -> Result<bool, XRPCError> {
        let state = self.get_campsite_permissions(flags)?;

        match state {
            PermissionState::Allowed => Ok(true),
            PermissionState::Inherit(leftovers) => {
                // Since it consumed one flag that was allowed, so the condition is satisfied
                if leftovers != flags {
                    Ok(true)
                } else {
                    self.with_roles.has_any_permissions(leftovers)
                }
            }
            PermissionState::Denied => Ok(false),
        }
    }
}
