use campground_lexicon::gg::campground::permission::{PermissionsDictionary, PermissionsStateDictionary};


#[derive(PartialEq, Eq, Debug)]
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
    pub fn map_non_denied<F: FnOnce() -> PermissionState>(self, map: F) -> PermissionState {
        match self {
            PermissionState::Denied => self,
            _ => map(),
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
    pub fn from_tent_three_level(bonfire: &PermissionsStateDictionary, category: &PermissionsStateDictionary, tent: &PermissionsStateDictionary, flag: i64) -> PermissionState {
        Self::from_tent(tent, flag)
            .map_inherit(||
                Self::from_tent(category, flag)
                    .map_inherit(||
                        Self::from_tent(bonfire, flag)
                    )
            )
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
    pub fn from_campsite_three_level(bonfire: &PermissionsStateDictionary, category: &PermissionsStateDictionary, tent: &PermissionsStateDictionary, flag: i64) -> PermissionState {
        Self::from_campsite(tent, flag)
            .map_inherit(||
                Self::from_campsite(category, flag)
                    .map_inherit(||
                        Self::from_campsite(bonfire, flag)
                    )
            )
    }
    pub fn is_allowed(self) -> bool {
        self == PermissionState::Allowed
    }
}