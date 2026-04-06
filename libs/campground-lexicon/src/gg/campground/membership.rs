use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::gg::campground::{actor::{ProfileViewBasic, ProfileViewDetailed}, role::RoleViewBasic};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberViewBasic {
    pub user: ProfileViewBasic,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberViewDetailed {
    pub user: ProfileViewDetailed,
    pub campsite_id: String,
    pub joined_at: String,
    pub used_invite_id: Option<Uuid>,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberViewAuthor {
    pub user: ProfileViewBasic,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
    pub is_member: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberBanView {
    pub user: ProfileViewBasic,
    pub user_id: String,
    pub campsite_id: String,
    pub reason: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMemberBansOutput {
    pub member_bans: Vec<MemberBanView>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyMemberRolesOutput {
    pub members: Vec<String>,
    pub role: RoleViewBasic,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteLeftOutput {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMembersOutput<T>
    where T: Serialize
{
    pub members: Vec<T>,
}

