use chrono::{DateTime, Utc};

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
pub enum Activity {
    #[serde(rename = "gg.campground.activity#custom")]
    Custom {
        emoji: Option<String>,
        state: Option<String>,
    },
    #[serde(rename = "gg.campground.activity#playing")]
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
    #[serde(rename = "gg.campground.activity#streaming")]
    #[serde(rename_all = "camelCase")]
    Streaming {
        name: String,
        url: String,
        created_at: DateTime<Utc>,
        details: Option<String>,
        assets: Option<ActivityAssets>,
    },
    #[serde(rename = "gg.campground.activity#listening")]
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
    #[serde(rename = "gg.campground.activity#watching")]
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
    #[serde(rename = "gg.campground.activity#competing")]
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