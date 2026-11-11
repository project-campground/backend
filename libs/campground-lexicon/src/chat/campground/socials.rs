#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename_all = "camelCase")]
pub enum SocialConnection {
    #[serde(rename = "chat.campground.socials#twitter")]
    #[serde(rename_all = "camelCase")]
    Twitter { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#reddit")]
    #[serde(rename_all = "camelCase")]
    Reddit { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#steam")]
    #[serde(rename_all = "camelCase")]
    Steam { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#youtube")]
    #[serde(rename_all = "camelCase")]
    Youtube { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#twitch")]
    #[serde(rename_all = "camelCase")]
    Twitch { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#github")]
    #[serde(rename_all = "camelCase")]
    Github { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#facebook")]
    #[serde(rename_all = "camelCase")]
    Facebook { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#tiktok")]
    #[serde(rename_all = "camelCase")]
    TikTok { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#instagram")]
    #[serde(rename_all = "camelCase")]
    Instagram { handle: String, user_id: String },
    #[serde(rename = "chat.campground.socials#mastodon")]
    #[serde(rename_all = "camelCase")]
    Mastodon {
        handle: String,
        instance: String,
        user_id: String,
    },
    #[serde(rename = "chat.campground.socials#bluesky")]
    #[serde(rename_all = "camelCase")]
    Bluesky { handle: String, did: String },
    #[serde(rename = "chat.campground.socials#roblox")]
    #[serde(rename_all = "camelCase")]
    Roblox {
        username: String,
        display_name: Option<String>,
        user_id: String,
    },
    #[serde(rename = "chat.campground.socials#website")]
    #[serde(rename_all = "camelCase")]
    Website { url: String },
}
