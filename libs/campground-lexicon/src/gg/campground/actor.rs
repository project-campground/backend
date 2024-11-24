use crate::gg::campground::socials::SocialConnection;
use rsky_lexicon::app::bsky::graph::ListViewBasic;
use rsky_lexicon::com::atproto::{
    label::{Label, SelfLabels},
    repo::Blob,
};
use chrono::{DateTime, Utc};

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
pub struct ActivityTimestamps {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityParty {
    pub id: Option<String>,
    pub current_size: Option<i32>,
    pub max_size: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityAssets {
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[non_exhaustive]
pub enum ProfileActivity {
    #[serde(rename = "gg.campground.actor.profileActivity#custom")]
    Custom {
        emoji: Option<String>,
        state: Option<String>,
    },
    #[serde(rename = "gg.campground.actor.profileActivity#playing")]
    #[serde(rename_all = "camelCase")]
    Playing {
        name: String,
        url: Option<String>,
        created_at: DateTime<Utc>,
        details: Option<String>,
        state: Option<String>,
        timestamps: Option<ActivityTimestamps>,
        party: Option<ActivityParty>,
        assets: Option<ActivityAssets>,
    },
    #[serde(rename = "gg.campground.actor.profileActivity#streaming")]
    #[serde(rename_all = "camelCase")]
    Streaming {
        name: String,
        url: String,
        created_at: DateTime<Utc>,
        details: Option<String>,
        assets: Option<ActivityAssets>,
    },
    #[serde(rename = "gg.campground.actor.profileActivity#listening")]
    #[serde(rename_all = "camelCase")]
    Listening {
        name: String,
        url: Option<String>,
        created_at: DateTime<Utc>,
        details: Option<String>,
        state: Option<String>,
        timestamps: Option<ActivityTimestamps>,
        party: Option<ActivityParty>,
        assets: Option<ActivityAssets>,
    },
    #[serde(rename = "gg.campground.actor.profileActivity#watching")]
    #[serde(rename_all = "camelCase")]
    Watching {
        name: String,
        url: Option<String>,
        created_at: DateTime<Utc>,
        details: Option<String>,
        state: Option<String>,
        timestamps: Option<ActivityTimestamps>,
        party: Option<ActivityParty>,
        assets: Option<ActivityAssets>,
    },
    #[serde(rename = "gg.campground.actor.profileActivity#competing")]
    #[serde(rename_all = "camelCase")]
    Competing {
        name: String,
        url: Option<String>,
        created_at: DateTime<Utc>,
        details: Option<String>,
        state: Option<String>,
        timestamps: Option<ActivityTimestamps>,
        party: Option<ActivityParty>,
        assets: Option<ActivityAssets>,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatus {
    pub activities: Vec<ProfileActivity>,
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
    pub activity: Option<ProfileActivity>,
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
    pub activities: Vec<ProfileActivity>,
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
    pub activities: Vec<ProfileActivity>,
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
    pub muted: Option<bool>,
    pub muted_by_list: Option<ListViewBasic>,
    pub blocked_by: Option<bool>,
    pub blocking_by_list: Option<ListViewBasic>,
}