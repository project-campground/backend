use std::collections::HashMap;

use uuid::Uuid;

use crate::gg::campground::{membership::CampsiteMemberViewBasic, permission::{PermissionsDictionary, PermissionsStateDictionary}, tent::{TentCategoryView, TentViewBasic}};

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
pub struct CampsiteRoleViewBasic {
    pub id: Uuid,
    pub campsite_id: String,
    
    pub name: String,
    pub display_separately: bool,
    pub mentionable: bool,
    pub flags: i32,

    pub permissions: PermissionsDictionary,
    
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

    pub bonfire_id: String,
    pub category_id: Option<Uuid>,
    pub tent_id: Option<Uuid>,

    pub user_id: Option<String>,
    pub role_id: Option<Uuid>,

    pub permissions: PermissionsStateDictionary,

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
pub struct GetCampsitePermissionsOutput {
    pub permissions: Vec<CampsitePermissionView>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampsiteRolesMovedOutput {
    pub roles_by_priority: HashMap<Uuid, i32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampsiteOutput {
    pub campsite: CampsiteViewDetailed,
    pub default_tent: TentViewBasic,
}
