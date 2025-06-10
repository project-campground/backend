use chrono::{DateTime, Utc};
use anyhow::{Result, bail};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JetstreamRepoCommitMessage {
    pub did: String,
    pub time_us: i64,
    pub kind: String,
    pub commit: JetstreamRepoCommit,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JetstreamRepoAccountMessage {
    pub did: String,
    pub time_us: i64,
    pub kind: String,
    pub account: JetstreamRepoAccount,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JetstreamRepoIdentityMessage {
    pub did: String,
    pub time_us: i64,
    pub kind: String,
    pub identity: JetstreamRepoIdentity,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JetstreamRepoCommit {
    pub rev: String,
    pub operation: String,
    pub collection: String,
    pub rkey: String,
    #[serde(rename = "record", skip_serializing_if = "Option::is_none")]
    pub record: Option<serde_json::Value>,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LikeSubject {
    pub cid: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JetstreamRepoIdentity {
    pub did: String,
    pub handle: String,
    pub seq: i64,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JetstreamRepoAccount {
    pub active: bool,
    pub did: String,
    pub seq: i64,
    pub time: DateTime<Utc>,
}

#[derive(Debug)]
pub enum JetstreamRepoMessage {
    Commit(Box<JetstreamRepoCommitMessage>),
    Identity(JetstreamRepoIdentityMessage),
    Account(JetstreamRepoAccountMessage),
}

pub fn read(data: &str) -> Result<JetstreamRepoMessage> {
    let data_json: serde_json::Value = serde_json::from_str(data)?;

    let binding = data_json.clone();
    let kind = binding["kind"].as_str().unwrap();

    let body = match kind {
        "commit" => JetstreamRepoMessage::Commit(serde_json::from_value(data_json)?),
        "account" => JetstreamRepoMessage::Account(serde_json::from_value(data_json)?),
        "identity" => JetstreamRepoMessage::Identity(serde_json::from_value(data_json)?),
        _ => {
            eprintln!("Received unknown kind {:?}", kind);
            bail!(format!("Received unknown kind {:?}", kind))
        }
    };

    Ok(body)
}