#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "chat.campground.homeServer")]
#[serde(rename_all = "camelCase")]
pub struct HomeServer {
    pub did: String,
}
