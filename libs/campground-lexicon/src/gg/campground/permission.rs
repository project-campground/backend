use uuid::Uuid;

use crate::gg::campground::{membership::MemberViewAuthor, role::RoleViewBasic};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionViewEmbedded {
    pub user_id: Option<String>,
    pub role_id: Option<Uuid>,

    pub permissions: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionViewDetailed {
    // pub id: Uuid,
    // pub campsite_id: String,
    pub bonfire_id: String,
    pub category_id: Option<Uuid>,
    pub tent_id: Option<Uuid>,

    pub member: Option<MemberViewAuthor>,
    pub role: Option<RoleViewBasic>,

    pub permissions: String,

    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPermissionsOutput {
    pub permissions: Vec<PermissionViewDetailed>,
}
