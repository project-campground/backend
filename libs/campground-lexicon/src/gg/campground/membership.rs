use uuid::Uuid;

use crate::gg::campground::{actor::{ProfileViewBasic, ProfileViewDetailed}, campsite::{CampsiteRoleViewBasic, CampsiteViewBasic}};


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteMemberViewBasic {
    pub user: ProfileViewBasic,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteMemberViewDetailed {
    pub user: ProfileViewDetailed,
    pub user_id: String,
    pub campsite_id: String,
    pub joined_at: String,
    pub used_invite_id: Option<String>,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteMemberViewAuthor {
    pub user: ProfileViewBasic,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
    pub is_member: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteInviteViewBasic {
    pub id: Uuid,
    pub campsite_id: String,
    pub allowed_amount: Option<i32>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub used: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteInviteViewDetailed {
    pub id: Uuid,
    pub campsite: CampsiteViewBasic,
    pub allowed_amount: Option<i32>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub used: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteBanView {
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
pub struct GetCampsiteInvitesOutput {
    pub invites: Vec<CampsiteInviteViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCampsiteBansOutput {
    pub bans: Vec<CampsiteBanView>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyMemberRolesOutput {
    pub members: Vec<String>,
    pub role: CampsiteRoleViewBasic,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteLeftOutput {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMembersOutput {
    pub members: Vec<CampsiteMemberViewBasic>,
}

