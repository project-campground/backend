#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "gg.campground.homeServer")]
#[serde(rename_all = "camelCase")]
pub struct HomeServer {
    pub did: String,
}