use uuid::Uuid;

use crate::gg::campground::{actor::ProfileViewBasic, tent::{TentCategoryView, TentViewBasic}};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteViewBasic {
    pub id: String,
    pub name: String,
    pub vanity_url: Option<String>,
    pub description: String,
    pub avatar_uri: Option<String>,
    pub banner_uri: Option<String>,
    pub tags: Vec<String>,

    pub member_count: usize,
    pub owner: String,

    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteViewDetailed {
    pub id: String,

    pub name: String,
    pub vanity_url: Option<String>,
    pub description: String,
    pub avatar_uri: Option<String>,
    pub banner_uri: Option<String>,
    pub tags: Vec<String>,

    pub member_count: usize,
    pub owner: String,

    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,

    pub bonfires: Vec<BonfireViewBasic>,
    pub roles: Vec<CampsiteRoleViewBasic>,
    pub member: CampsiteMemberViewBasic,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteMemberViewBasic {
    pub user: ProfileViewBasic,
    pub user_id: String,
    pub campsite_id: String,
    pub joined_at: String,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
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
pub struct CampsiteMemberViewDetailed {
    pub user_id: String,
    pub campsite_id: String,
    pub joined_at: String,
    pub used_invite_id: Option<String>,
    pub nickname: Option<String>,
    pub roles: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteRoleViewBasic {
    pub id: Uuid,
    pub campsite_id: String,
    
    pub name: String,
    pub display_separately: bool,
    pub mentionable: bool,
    pub flags: i32,

    pub campsite_permissions: i64,
    pub tent_permissions: i64,
    
    pub priority: i32,
    
    pub color: i32,
    pub color_secondary: i32,
    
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsitePermissionView {
    pub id: Uuid,
    pub campsite_id: String,

    pub bonfire_id: Option<String>,
    pub category_id: Option<Uuid>,
    pub tent_id: Option<Uuid>,

    pub user_id: Option<String>,
    pub role_id: Option<Uuid>,

    pub allowed_campsite_permissions: i64,
    pub denied_campsite_permissions: i64,
    pub allowed_tent_permissions: i64,
    pub denied_tent_permissions: i64,

    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BonfireViewBasic {
    pub id: String,
    pub campsite_id: String,
    pub name: String,
    pub description: String,
    pub avatar_uri: Option<String>,
    pub banner_uri: Option<String>,
    pub priority: i32,
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BonfireViewDetailed {
    pub id: String,
    pub campsite_id: String,
    pub name: String,
    pub description: String,
    pub avatar_uri: Option<String>,
    pub banner_uri: Option<String>,
    pub priority: i32,
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
    pub tents: Vec<TentViewBasic>,
    pub categories: Vec<TentCategoryView>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCampsitesOutput {
    pub campsites: Vec<CampsiteViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCampsiteRolesOutput {
    pub roles: Vec<CampsiteRoleViewBasic>,
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
pub struct GetMembersOutput {
    pub members: Vec<CampsiteMemberViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampsiteOutput {
    pub campsite: CampsiteViewDetailed,
    pub default_tent: TentViewBasic,
}
