use crate::app::bsky::actor::ProfileViewBasic;
use crate::com::atproto::label::Label;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "app.bsky.labeler.defs#labelerView")]
#[serde(rename_all = "camelCase")]
pub struct LabelerView {
    pub uri: String,
    pub cid: String,
    pub creator: ProfileViewBasic,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<LabelerViewerState>,
    pub indexed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<Label>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LabelerViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like: Option<String>,
}
