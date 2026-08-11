use appview_schema::models::appview::Actor;
use atproto_identity::{
    model::Document, plc, storage::DidDocumentStorage, storage_lru::LruDidDocumentStorage, web,
};
use reqwest::Client;

pub enum DidDocIndexingError {
    InternalError,
    UnknownMethod(String),
    HandleNotFound,
    #[allow(unused)]
    DocNotFound,
}

impl std::fmt::Display for DidDocIndexingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DidDocIndexingError::HandleNotFound => {
                f.write_str("DidDocIndexingError::HandleNotFound")
            }
            DidDocIndexingError::InternalError => f.write_str("DidDocIndexingError::InternalError"),
            DidDocIndexingError::DocNotFound => write!(f, "DidDocIndexingError::DocNotFound"),
            DidDocIndexingError::UnknownMethod(method) => {
                write!(f, "DidDocIndexingError::UnknownMethod({})", method)
            }
        }
    }
}

pub async fn index_actor(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    did: &str,
) -> Result<(Actor, Document), DidDocIndexingError> {
    let doc = get_actor_did_doc(client, did_document_storage, did).await?;

    println!("Doc");
    println!("Doc: {:?}", doc);

    let handle = doc
        .also_known_as
        .iter()
        .find(|&handle| handle.starts_with("at://"))
        .map(|value| value.split('/').into_iter().nth(2))
        .flatten()
        .map(|value| value.to_string());

    if handle.is_none() {
        return Err(DidDocIndexingError::HandleNotFound);
    }

    let actor = Actor {
        did: did.to_string(),
        handle: handle,
        indexed_at: chrono::Utc::now().naive_utc().to_string(),
        campsites: vec![],
    };

    Ok((actor, doc))
}

fn handle_unknown_index_error<T>(err: T) -> DidDocIndexingError
where
    T: std::fmt::Display,
{
    println!("Unknown error while indexing actor: {}", err);
    DidDocIndexingError::InternalError
}

/// # Summary
/// Gets cached actor's DID Document or fetches the actor by the provided DID. This does not resolve actor's handle.
pub async fn get_actor_did_doc(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    did: &str,
) -> Result<Document, DidDocIndexingError> {
    let did_doc_cached = did_document_storage
        .get_document_by_did(&did)
        .await
        .map_err(handle_unknown_index_error)?;

    let doc = if let Some(cached_doc) = did_doc_cached {
        cached_doc
    } else {
        fetch_actor_did_doc(client, did_document_storage, &did).await?
    };

    Ok(doc)
}

async fn fetch_actor_did_doc(
    client: &Client,
    did_document_storage: &LruDidDocumentStorage,
    did: &str,
) -> Result<Document, DidDocIndexingError> {
    let fetched_doc = match *did.split(":").collect::<Vec<&str>>().get(1).unwrap() {
        "plc" => plc::query(client, "plc.directory", did)
            .await
            .map_err(handle_unknown_index_error)?,
        "web" => web::query(client, &did)
            .await
            .map_err(handle_unknown_index_error)?,
        method => return Err(DidDocIndexingError::UnknownMethod(method.to_string())),
    };

    did_document_storage
        .store_document(fetched_doc.clone())
        .await
        .map_err(handle_unknown_index_error)?;
    Ok(fetched_doc)
}
