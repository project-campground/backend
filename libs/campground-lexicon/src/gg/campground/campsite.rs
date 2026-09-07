use crate::gg::campground::{
    bonfire::BonfireViewBasic, membership::MemberViewBasic, role::RoleViewBasic,
    tent::TentViewBasic,
};

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
    pub roles: Vec<RoleViewBasic>,
    pub me: MemberViewBasic,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCampsitesOutput {
    pub campsites: Vec<CampsiteViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampsiteOutput {
    pub campsite: CampsiteViewDetailed,
    pub default_tent: TentViewBasic,
}
