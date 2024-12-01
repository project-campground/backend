#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename_all = "camelCase")]
pub enum SocialConnection {
    #[serde(rename = "gg.campground.socials.defs#twitter")]
    Twitter {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#reddit")]
    Reddit {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#steam")]
    Steam {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#youtube")]
    Youtube {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#twitch")]
    Twitch {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#github")]
    Github {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#facebook")]
    Facebook {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#tiktok")]
    TikTok {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#instagram")]
    Instagram {
        handle: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#mastodon")]
    Mastodon {
        handle: String,
        instance: String,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#bluesky")]
    Bluesky {
        handle: String,
        did: String,
    },
    #[serde(rename = "gg.campground.socials.defs#roblox")]
    Roblox {
        username: String,
        display_name: Option<String>,
        user_id: String,
    },
    #[serde(rename = "gg.campground.socials.defs#website")]
    Website {
        url: String,
    },
}