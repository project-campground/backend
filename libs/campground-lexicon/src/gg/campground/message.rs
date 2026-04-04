use uuid::Uuid;

use crate::gg::campground::{content::ContentComponent, membership::MemberViewAuthor};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MessageType {
    Default = 0,
    System = 1,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageViewBasic {
    pub id: Uuid,
    pub campsite_id: String,
    pub bonfire_id: String,
    pub tent_id: Uuid,

    pub r#type: Option<MessageType>,
    pub content: String,
    pub components: Option<Vec<ContentComponent>>,
    pub replying_to: Vec<Uuid>,
    
    // pub created_by: String,
    pub created_by: MemberViewAuthor,
    
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageViewWithReplies {
    pub id: Uuid,
    pub campsite_id: String,
    pub bonfire_id: String,
    pub tent_id: Uuid,
    
    pub r#type: Option<MessageType>,
    pub content: String,
    pub replying_to: Vec<MessageViewBasic>,
    pub replying_to_count: usize,
    pub components: Option<Vec<ContentComponent>>,
    
    // pub created_by: String,
    pub created_by: MemberViewAuthor,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesOutput {
    pub messages: Vec<MessageViewWithReplies>,
}
