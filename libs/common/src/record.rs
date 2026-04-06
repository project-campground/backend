use anyhow::Result;
use atproto_identity::{resolve::resolve_subject, storage_lru::LruDidDocumentStorage};
use hickory_resolver::TokioResolver;
use reqwest::Client;
use rsky_lexicon::com::atproto::repo::Blob;
use serde::Deserialize;

use crate::fetch_did_document;

#[derive(Deserialize)]
pub struct GetRecordResponse<T> {
    pub uri: String,
    pub cid: String,
    pub value: T
}

#[derive(Deserialize)]
pub struct GetRecordListResponse<T> {
    pub records: Vec<GetRecordResponse<T>>,
    pub cursor: Option<String>,
}

pub async fn fetch_record<'a, T>(
    actor: &str,
    collection: &str,
    rkey: &str,
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    dns_resolver: &TokioResolver
) -> Result<GetRecordResponse<T>>
where T: for<'de> Deserialize<'de> + 'a
{
    let actor = resolve_subject(client, dns_resolver, actor).await?;
    let did_document = fetch_did_document(&actor, dns_resolver, did_document_storage, client).await?;

    let pds_service = did_document
        .service
        .iter()
        .find(
            |service| service.id == "#atproto_pds" && service.r#type == "AtprotoPersonalDataServer"
        )
        .ok_or_else(|| anyhow::anyhow!("PDS service not found"))?;
    let endpoint = &pds_service.service_endpoint;
    let url = format!("{endpoint}/xrpc/com.atproto.repo.getRecord?repo={actor}&collection={collection}&rkey={rkey}");
    Ok(client
        .get(url)
        .send()
        .await?
        .json::<GetRecordResponse<T>>()
        .await?)
}

pub fn get_blob_ref(blob: &Option<Blob>) -> Option<String> {
    match blob {
        Some(blob) => Some(blob.r#ref.unwrap().to_string()),
        None => None
    }
}

pub async fn fetch_record_list<'a, T>(
    actor: &str,
    collection: &str,
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    dns_resolver: &TokioResolver
) -> Result<GetRecordListResponse<T>>
where T: for<'de> Deserialize<'de> + 'a
{
    let actor = resolve_subject(client, dns_resolver, actor).await?;
    let did_document = fetch_did_document(&actor, dns_resolver, did_document_storage, client).await?;

    let pds_service = did_document
        .service
        .iter()
        .find(
            |service| service.id == "#atproto_pds" && service.r#type == "AtprotoPersonalDataServer"
        )
        .ok_or_else(|| anyhow::anyhow!("PDS service not found"))?;
    let endpoint = &pds_service.service_endpoint;
    let url = format!("{endpoint}/xrpc/com.atproto.repo.listRecords?repo={actor}&collection={collection}");
    Ok(client
        .get(url)
        .send()
        .await?
        .json::<GetRecordListResponse<T>>()
        .await?)
}
