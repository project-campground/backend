use crate::gg::campground::actor::ProfileViewBasic;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "gg.campground.profile.post")]
#[serde(rename_all = "camelCase")]
pub struct ProfilePost {
    pub content: Option<String>,
    pub parent_uri: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePostViewBasic {
    pub cid: String,
    pub uri: String,
    pub parent_uri: Option<String>,
    pub content: String,
    pub author: ProfileViewBasic,
    pub reply_count: usize,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub indexed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePostViewParented {
    pub cid: String,
    pub uri: String,
    pub parent_uri: Option<String>,
    pub content: String,
    pub author: ProfileViewBasic,
    pub reply_count: usize,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub indexed_at: Option<String>,
    pub parent: Option<ProfilePostViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePostViewDetailed {
    pub cid: String,
    pub uri: String,
    pub parent_uri: Option<String>,
    pub content: String,
    pub author: ProfileViewBasic,
    pub replies: Vec<ProfilePostViewBasic>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub indexed_at: Option<String>,
    pub parent: Option<ProfilePostViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetProfilePostsOutput {
    pub posts: Vec<ProfilePostViewParented>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetProfilePostRepliesOutput {
    pub posts: Vec<ProfilePostViewBasic>,
}
