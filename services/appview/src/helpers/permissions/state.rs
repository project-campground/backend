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
    pub fn from_role_content(permissions: &PermissionsDictionary, flag: u64) -> PermissionState {
        if permissions.content & flag == flag { PermissionState::Allowed } else { PermissionState::Denied }
    }
    #[allow(dead_code)]
    pub fn from_role_general(permissions: &PermissionsDictionary, flag: u64) -> PermissionState {
        if permissions.general & flag == flag { PermissionState::Allowed } else { PermissionState::Denied }
    }
    pub fn from_content_optional(permission: Option<&PermissionsStateDictionary>, flag: u64) -> PermissionState {
        permission.map_or(PermissionState::Inherit, |x| PermissionState::from_content(x, flag))
    }
    pub fn from_content(permission: &PermissionsStateDictionary, flag: u64) -> PermissionState {
        if permission.allowed.content & flag == flag {
            PermissionState::Allowed
        } else if permission.denied.content & flag == flag {
            PermissionState::Denied
        } else {
            PermissionState::Inherit
        }
    }
    pub fn from_content_three_level(bonfire: &PermissionsStateDictionary, category: &PermissionsStateDictionary, tent: &PermissionsStateDictionary, flag: u64) -> PermissionState {
        Self::from_content(tent, flag)
            .map_inherit(||
                Self::from_content(category, flag)
                    .map_inherit(||
                        Self::from_content(bonfire, flag)
                    )
            )
    }
    #[allow(dead_code)]
    pub fn from_general_optional(permission: Option<&PermissionsStateDictionary>, flag: u64) -> PermissionState {
        permission.map_or(PermissionState::Inherit, |x| PermissionState::from_general(x, flag))
    }
    pub fn from_general(permission: &PermissionsStateDictionary, flag: u64) -> PermissionState {
        if permission.allowed.general & flag == flag {
            PermissionState::Allowed
        } else if permission.denied.general & flag == flag {
            PermissionState::Denied
        } else {
            PermissionState::Inherit
        }
    }
    pub fn from_general_three_level(bonfire: &PermissionsStateDictionary, category: &PermissionsStateDictionary, tent: &PermissionsStateDictionary, flag: u64) -> PermissionState {
        Self::from_general(tent, flag)
            .map_inherit(||
                Self::from_general(category, flag)
                    .map_inherit(||
                        Self::from_general(bonfire, flag)
                    )
            )
    }
    pub fn is_allowed(self) -> bool {
        self == PermissionState::Allowed
    }
}