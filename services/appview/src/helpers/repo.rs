use reqwest::Client;
use rocket::State;
use rsky_lexicon::com::atproto::repo::Blob;

use crate::xrpc_server::{error::{Result, XRPCError}, GetRecordResponse};

pub async fn get_record<T>(client: &State<Client>, endpoint: &str, did: &str, collection: &str, rkey: &str) -> Option<GetRecordResponse<T>>
    where T: serde::de::DeserializeOwned + 'static
{
    match client.get(format!("{endpoint}/xrpc/com.atproto.repo.getRecord?repo={did}&collection={collection}&rkey={rkey}"))
    .send()
    .await {
        Ok(res) => {
            if !res.status().is_success() {
                return None;
            }
            match res.json::<GetRecordResponse<T>>().await {
                Ok(record) => Some(record),
                Err(_) => None
            }
        },
        Err(_) => None
    }
}

pub async fn try_get_record<T>(client: &State<Client>, endpoint: &str, did: &str, collection: &str, rkey: &str) -> Result<GetRecordResponse<T>>
    where T: serde::de::DeserializeOwned + 'static
{
    match get_record::<T>(client, endpoint, did, collection, rkey).await {
        Some(record) => Ok(record),
        None => Err(XRPCError::NotFound)
    }
}

pub fn get_blob_ref(blob: &Option<Blob>) -> Option<String> {
    match blob {
        Some(blob) => Some(blob.r#ref.unwrap().to_string()),
        None => None
    }
}