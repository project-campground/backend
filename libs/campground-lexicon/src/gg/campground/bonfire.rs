use crate::gg::campground::tent::{TentCategoryView, TentViewBasic};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BonfireViewBasic {
    pub id: String,
    pub campsite_id: String,
    pub name: String,
    pub description: String,
    pub avatar_uri: Option<String>,
    pub banner_uri: Option<String>,
    pub position: i32,
    pub home: bool,
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
    pub home: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
    pub tents: Vec<TentViewBasic>,
    pub categories: Vec<TentCategoryView>,
}
