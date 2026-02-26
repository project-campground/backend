use uuid::Uuid;

use crate::gg::campground::{campsite::CampsitePermissionView, membership::CampsiteMemberViewAuthor};

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
    pub view_type: i32,

    pub priority: i32,
    
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
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
    pub view_type: i32,

    pub priority: i32,

    pub permissions: Vec<CampsitePermissionView>,

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

    pub priority: i32,
    
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TentMessageViewBasic {
    pub id: Uuid,
    pub campsite_id: String,
    pub bonfire_id: String,
    pub tent_id: Uuid,
    
    pub content: String,
    pub replying_to: Vec<Uuid>,
    
    // pub created_by: String,
    pub created_by: CampsiteMemberViewAuthor,

    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TentMessageViewWithReplies {
    pub id: Uuid,
    pub campsite_id: String,
    pub bonfire_id: String,
    pub tent_id: Uuid,
    
    pub content: String,
    pub replying_to: Vec<TentMessageViewBasic>,
    pub replying_to_count: usize,
    
    // pub created_by: String,
    pub created_by: CampsiteMemberViewAuthor,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTentsOutput {
    pub permissions: Vec<CampsitePermissionView>,
    pub tents: Vec<TentViewBasic>,
    pub categories: Vec<TentCategoryView>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTentMessagesOutput {
    pub messages: Vec<TentMessageViewWithReplies>,
}
