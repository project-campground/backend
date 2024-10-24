#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: Thing,
    pub created_at: DateTime<Utc>,
    pub email: String,
    pub password: String,
    pub email_confirmed_at: Option<DateTime<Utc>>,
}

impl Account {
    pub fn did(&self) -> String {
        self.id.id.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoRoot {
    pub id: Thing,
    pub cid: String,
    pub rev: String,
    pub indexed_at: DateTime<Utc>,
}

impl RepoRoot {
    pub fn did(&self) -> String {
        self.id.id.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoBlock {
    pub did: String,
    pub cid: String,
    pub repo_rev: String,
    pub size: i32,
    pub content: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: Thing,
    pub did: String,
    pub cid: String,
    pub collection: String,
    pub rkey: String,
    pub repo_rev: String,
    pub indexed_at: DateTime<Utc>,
    pub takedown_ref: Option<String>,
}

impl Record {
    pub fn uri(&self) -> String {
        self.id.id.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    pub id: Thing,
    pub did: String,
    pub cid: String,
    pub mime_type: String,
    pub size: i32,
    pub temp_key: Option<String>,
    pub width: i32,
    pub height: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordBlob {
    pub id: Thing,
    pub record_uri: String,
    pub blob_uri: String,
    pub did: String,
}