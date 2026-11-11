use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum ContentComponent {
    System(SystemMessage)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "message")]
#[serde(rename_all = "camelCase")]
pub enum SystemMessage {
    #[serde(rename_all = "camelCase")]
    TentCreated {
        tent_name: String,
    },
    #[serde(rename_all = "camelCase")]
    TentNameUpdated {
        previous_name: String,
        new_name: String,
    },
}