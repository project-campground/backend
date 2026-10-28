// This is stupid, but I am not repeating it
// Total of 32 flags available (64 bits / 2 bit flag)
// This eliminates the need to calculate this fucking bullshit every single time
pub const INVALID_PERMISSION_FLAG_MASK: u64 = {
    let mut value: u64 = 0;
    let mut i = 0;

    // Since for i in 0..62.step_by(2) can't be used in consts
    loop {
        value |= 0b01 << i;

        i += 2;
        if i >= 64 {
            break;
        }
    }

    value
};

#[derive(PartialEq, Eq, Debug)]
pub enum PermissionState {
    Allowed,
    Inherit(u32),
    Denied,
}

impl PermissionState {
    pub fn map_inherit<F: FnOnce(u32) -> PermissionState>(self, map: F) -> PermissionState {
        match self {
            PermissionState::Inherit(value) => map(value),
            _ => self,
        }
    }
    pub fn map_non_denied<F: FnOnce() -> PermissionState>(self, map: F) -> PermissionState {
        match self {
            PermissionState::Denied => self,
            _ => map(),
        }
    }
    pub fn from_role(permissions: &u32, flag: u32) -> PermissionState {
        if permissions & flag == flag {
            PermissionState::Allowed
        } else {
            PermissionState::Denied
        }
    }
    pub fn from_optional(permission: Option<&u64>, flag: u32) -> PermissionState {
        permission.map_or(PermissionState::Inherit(flag), |x| {
            PermissionState::from(x, flag)
        })
    }
    pub fn from_single(permission: &u64, flag: u32) -> PermissionState {
        let state_pos = (31 - flag.leading_zeros()) * 2;
        let bitmask = 0b11 << state_pos;
        let value = (permission & bitmask) >> state_pos;

        if value == 0b11 {
            PermissionState::Allowed
        } else if value == 0b10 {
            PermissionState::Denied
        } else {
            PermissionState::Inherit(flag)
        }
    }
    pub fn from(permission: &u64, flags: u32) -> PermissionState {
        let mut missing = 0u32;

        for pos in flags.trailing_zeros()..(32 - flags.leading_zeros()) {
            let state = (permission >> (pos * 2)) & 0b11;

            if state == 0b11 {
                continue;
            } else if state == 0b10 {
                return PermissionState::Denied;
            } else {
                missing |= 1 >> pos;
            }
        }

        if missing == 0 {
            PermissionState::Allowed
        } else {
            PermissionState::Inherit(missing)
        }
    }
    pub fn from_three_level(
        bonfire: &u64,
        category: &u64,
        tent: &u64,
        flag: u32,
    ) -> PermissionState {
        Self::from(tent, flag).map_inherit(|flag| {
            Self::from(category, flag).map_inherit(|flag| Self::from(bonfire, flag))
        })
    }
    pub fn is_allowed(self) -> bool {
        self == PermissionState::Allowed
    }
}
