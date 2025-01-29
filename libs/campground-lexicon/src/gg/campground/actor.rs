use crate::gg::campground::socials::SocialConnection;
use rsky_lexicon::com::atproto::{
    label::{Label, SelfLabels},
    repo::Blob,
};
use chrono::{DateTime, Utc};
use crate::gg::campground::activity::Activity;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ProfileStatusType {
    Online,
    DoNotDisturb,
    Idle,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatus {
    pub activities: Vec<Activity>,
    pub status_type: Option<ProfileStatusType>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "gg.campground.actor.profile")]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub display_name: Option<String>,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub social_connections: Option<Vec<SocialConnection>>,
    /// Small image to be displayed next to posts from account. AKA, 'profile picture'
    pub avatar: Option<Blob>,
    /// Larger horizontal image to display behind profile view.
    pub banner: Option<Blob>,
    pub labels: Option<ProfileLabels>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum ProfileLabels {
    #[serde(rename = "com.atproto.label.defs#selfLabels")]
    SelfLabels(SelfLabels),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewBasic {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub status: Option<ProfileStatus>,
    pub activity: Option<Activity>,
    pub avatar: Option<String>,
    pub viewer: Option<ViewerState>,
    pub labels: Option<Vec<Label>>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub status: Option<ProfileStatus>,
    pub activities: Vec<Activity>,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub labels: Vec<Label>,
    pub indexed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewDetailed {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub status: Option<ProfileStatus>,
    pub activities: Vec<Activity>,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub social_connections: Option<Vec<SocialConnection>>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub viewer: Option<ViewerState>,
    pub labels: Vec<Label>,
    pub indexed_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetProfilesOutput {
    pub profiles: Vec<ProfileViewDetailed>,
}

/// Metadata about the requesting account's relationship with the subject account.
/// Only has meaningful content for authed requests.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerState {
    pub blocked_by: Option<bool>,
    pub blocking: Option<String>,
}