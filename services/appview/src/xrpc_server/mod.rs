use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetRecordResponse<T> {
    pub uri: String,
    pub cid: String,
    pub value: T
}

pub mod error;
pub mod auth;