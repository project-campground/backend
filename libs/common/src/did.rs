use anyhow::{bail, Result};
use atproto_identity::{model::{Document, Service}, plc, resolve::resolve_subject, storage::DidDocumentStorage, storage_lru::LruDidDocumentStorage, web};
use hickory_resolver::TokioResolver;
use reqwest::Client;

pub async fn fetch_did_document(actor: &str, dns_resolver: &TokioResolver, did_document_storage: &LruDidDocumentStorage, client: &Client) -> Result<Document> {
    let actor = resolve_subject(client, dns_resolver, actor).await?;
    let mut did_document = did_document_storage.get_document_by_did(&actor).await?;

    if did_document.is_none() {
        match *actor.split(":").collect::<Vec<&str>>().get(1).unwrap() {
            "plc" => {
                did_document = plc::query(client, "plc.directory", &actor).await.ok()
            },
            "web" => {
                did_document = web::query(client, &actor).await.ok()
            },
            _ => bail!("Unsupported DID method")
        }
    }
    did_document.ok_or(anyhow::anyhow!("Failed to fetch DID document"))
}

pub fn get_handle(doc: &Document) -> Option<String> {
    doc.also_known_as.iter().find(|&s| s.starts_with("at://")).map(|s| s.replace("at://", ""))
}

pub fn get_service(doc: &Document, id: String, r#type: String) -> Option<Service> {
    doc.service.iter().find(|s| s.id == id && s.r#type == r#type).cloned()
}