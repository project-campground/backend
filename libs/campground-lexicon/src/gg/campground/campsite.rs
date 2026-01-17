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
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
    pub bonfires: Vec<BonfireViewBasic>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteMemberViewBasic {
    pub user: ProfileViewBasic,
    pub user_id: String,
    pub campsite_id: String,
    pub joined_at: String,
    pub nickname: Option<String>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteMemberViewDetailed {
    pub user_id: String,
    pub campsite_id: String,
    pub joined_at: String,
    pub used_invite_id: Option<String>,
    pub nickname: Option<String>
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
pub struct GetMembersOutput {
    pub members: Vec<CampsiteMemberViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampsiteOutput {
    pub campsite: CampsiteViewDetailed,
    pub default_tent: TentViewBasic,
    pub owner_member: CampsiteMemberViewBasic
}
