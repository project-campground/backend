#[macro_export]
macro_rules! expect_permission {
    ($expr: expr) => {
        if !$expr {
            use crate::xrpc::error::XRPCError;
            return Err(XRPCError::Forbidden(
                "No given permission to do that".to_string(),
            ));
        }
    };
}

pub struct PermissionConsts {}
impl PermissionConsts {
    // Campsite management
    pub const MANAGE_CAMPSITE: u32 = 1 << 0;
    pub const MANAGE_BONFIRES: u32 = 1 << 1;
    pub const MANAGE_TENTS: u32 = 1 << 2;
    pub const MANAGE_ROLES: u32 = 1 << 3;
    // Membership
    pub const GIVE_ROLES: u32 = 1 << 4;
    pub const MUTE_MEMBERS: u32 = 1 << 5;
    pub const KICK_MEMBERS: u32 = 1 << 6;
    pub const BAN_MEMBERS: u32 = 1 << 7;
    // Identity
    pub const MANAGE_SELF_IDENTITY: u32 = 1 << 8;
    pub const MANAGE_OTHERS_IDENTITY: u32 = 1 << 9;
    // Invites
    pub const CREATE_INVITES: u32 = 1 << 10;
    pub const MANAGE_INVITES: u32 = 1 << 11;
    // Content
    pub const VIEW_CONTENT: u32 = 1 << 16;
    pub const CREATE_CONTENT: u32 = 1 << 17;
    #[allow(dead_code)]
    pub const PIN_CONTENT: u32 = 1 << 18;
    pub const MANAGE_CONTENT: u32 = 1 << 19;
    #[allow(dead_code)]
    pub const MENTION_EVERYONE: u32 = 1 << 20;
    #[allow(dead_code)]
    pub const CREATE_PRIVATE_CONTENT: u32 = 1 << 21;

    #[allow(dead_code)]
    pub const MAX_CAMPSITE: u32 = 0b1111;
    #[allow(dead_code)]
    pub const MAX_MEMBERSHIP: u32 = 0b1111 << 4;
    #[allow(dead_code)]
    pub const MAX_IDENTITY: u32 = 0b11 << 8;
    #[allow(dead_code)]
    pub const MAX_CONTENT: u32 = 0b111111 << 16;
    #[allow(dead_code)]
    pub const MAX: u32 = PermissionConsts::MAX_CAMPSITE
        | PermissionConsts::MAX_MEMBERSHIP
        | PermissionConsts::MAX_IDENTITY
        | PermissionConsts::MAX_CONTENT;
}

pub mod check;
pub mod fetch;
pub mod state;
