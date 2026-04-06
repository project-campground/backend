use uuid::Uuid;

use crate::gg::campground::permission::PermissionViewBasic;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum TentType {
    Text = 0,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TentViewBasic {
    pub id: Uuid,
    pub campsite_id: String,
    pub bonfire_id: String,
    pub category_id: Option<Uuid>,

    pub name: String,
    pub description: String,
    pub r#type: TentType,
    pub view_type: i16,

    pub position: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TentViewDetailed {
    pub id: Uuid,
    pub campsite_id: String,
    pub bonfire_id: String,
    pub category_id: Option<Uuid>,
    
    pub name: String,
    pub description: String,
    pub r#type: TentType,
    pub view_type: i16,

    pub position: i32,

    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TentCategoryView {
    pub id: Uuid,
    pub campsite_id: String,
    pub bonfire_id: String,
    
    pub name: String,
    pub description: String,

    pub position: i32,
    
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTentsOutput {
    pub permissions: Vec<PermissionViewBasic>,
    pub tents: Vec<TentViewBasic>,
    pub categories: Vec<TentCategoryView>,
}
