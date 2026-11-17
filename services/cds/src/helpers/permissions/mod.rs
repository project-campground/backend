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

pub struct PermissionPosition {}
impl PermissionPosition {
    // Campsite management
    pub const MANAGE_CAMPSITE: u8 = 0;
    pub const MANAGE_BONFIRES: u8 = 1;
    pub const MANAGE_TENTS: u8 = 2;
    pub const MANAGE_ROLES: u8 = 3;
    // Membership
    pub const GIVE_ROLES: u8 = 4;
    pub const MUTE_MEMBERS: u8 = 5;
    pub const KICK_MEMBERS: u8 = 6;
    pub const BAN_MEMBERS: u8 = 7;
    // Identity
    pub const MANAGE_SELF_IDENTITY: u8 = 8;
    pub const MANAGE_OTHERS_IDENTITY: u8 = 9;
    // Invites
    pub const CREATE_INVITES: u8 = 10;
    pub const MANAGE_INVITES: u8 = 11;
    // Content
    pub const VIEW_CONTENT: u8 = 16;
    pub const CREATE_CONTENT: u8 = 17;
    pub const PIN_CONTENT: u8 = 18;
    pub const MANAGE_CONTENT: u8 = 19;
    pub const MENTION_EVERYONE: u8 = 20;
    pub const CREATE_PRIVATE_CONTENT: u8 = 21;
}
pub struct PermissionFlag {}
impl PermissionFlag {
    // Campsite management
    pub const MANAGE_CAMPSITE: u32 = 1 << PermissionPosition::MANAGE_CAMPSITE;
    pub const MANAGE_BONFIRES: u32 = 1 << PermissionPosition::MANAGE_BONFIRES;
    pub const MANAGE_TENTS: u32 = 1 << PermissionPosition::MANAGE_TENTS;
    pub const MANAGE_ROLES: u32 = 1 << PermissionPosition::MANAGE_ROLES;
    // Membership
    pub const GIVE_ROLES: u32 = 1 << PermissionPosition::GIVE_ROLES;
    pub const MUTE_MEMBERS: u32 = 1 << PermissionPosition::MUTE_MEMBERS;
    pub const KICK_MEMBERS: u32 = 1 << PermissionPosition::KICK_MEMBERS;
    pub const BAN_MEMBERS: u32 = 1 << PermissionPosition::BAN_MEMBERS;
    // Identity
    pub const MANAGE_SELF_IDENTITY: u32 = 1 << PermissionPosition::MANAGE_SELF_IDENTITY;
    pub const MANAGE_OTHERS_IDENTITY: u32 = 1 << PermissionPosition::MANAGE_OTHERS_IDENTITY;
    // Invites
    pub const CREATE_INVITES: u32 = 1 << PermissionPosition::CREATE_INVITES;
    pub const MANAGE_INVITES: u32 = 1 << PermissionPosition::MANAGE_INVITES;
    // Content
    pub const VIEW_CONTENT: u32 = 1 << PermissionPosition::VIEW_CONTENT;
    pub const CREATE_CONTENT: u32 = 1 << PermissionPosition::CREATE_CONTENT;
    #[allow(dead_code)]
    pub const PIN_CONTENT: u32 = 1 << PermissionPosition::PIN_CONTENT;
    pub const MANAGE_CONTENT: u32 = 1 << PermissionPosition::MANAGE_CONTENT;
    #[allow(dead_code)]
    pub const MENTION_EVERYONE: u32 = 1 << PermissionPosition::MENTION_EVERYONE;
    #[allow(dead_code)]
    pub const CREATE_PRIVATE_CONTENT: u32 = 1 << PermissionPosition::CREATE_PRIVATE_CONTENT;

    #[allow(dead_code)]
    pub const MAX_CAMPSITE: u32 = 0b1111;
    #[allow(dead_code)]
    pub const MAX_MEMBERSHIP: u32 = 0b1111 << 4;
    #[allow(dead_code)]
    pub const MAX_IDENTITY: u32 = 0b11 << 8;
    #[allow(dead_code)]
    pub const MAX_CONTENT: u32 = 0b111111 << 16;
    #[allow(dead_code)]
    pub const MAX: u32 = PermissionFlag::MAX_CAMPSITE
        | PermissionFlag::MAX_MEMBERSHIP
        | PermissionFlag::MAX_IDENTITY
        | PermissionFlag::MAX_CONTENT;
}

pub mod check;
pub mod fetch;
pub mod state;
