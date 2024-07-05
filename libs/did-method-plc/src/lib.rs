extern crate thiserror;

use operation::{SignedPLCOperation, UnsignedPLCOperation};
use didkit::{
    DIDMethod, DIDResolver, Document, DocumentMetadata, ResolutionInputMetadata, ResolutionMetadata,
};
use async_trait::async_trait;

pub mod operation;
mod multicodec;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
pub const DEFAULT_HOST: &str = "https://plc.directory";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("DAG-CBOR error: {0}")]
    DagCbor(String),

    #[error("Operation is unsigned")]
    UnsignedOperation,

    #[error("Invalid key")]
    InvalidKey,

    #[error("Multibase error")]
    Multibase(#[from] multibase::Error),

    #[error("ECDSA signature error: {0}")]
    Signature(#[from] ecdsa::signature::Error),

    #[error("Sec1 error: {0}")]
    Sec1(#[from] sec1::Error),

    #[error("Hex error")]
    Hex(#[from] hex::FromHexError),

    #[error("Multicodec error: {0}")]
    Multicodec(#[from] multicodec::Error),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

/// did:plc Method
///
/// [Specification](https://github.com/did-method-plc/did-method-plc#did-plc-method-didplc)
pub struct DIDPLC {
    host: String,
    client: reqwest::Client,
}

impl DIDPLC {
    pub fn new(host: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap();

        Self {
            host: host.to_string(),
            client,
        }
    }
}

impl Default for DIDPLC {
    fn default() -> Self {
        Self::new(DEFAULT_HOST)
    }
}

impl DIDMethod for DIDPLC {
    fn name(&self) -> &'static str {
        "did:plc"
    }

    fn to_resolver(&self) -> &dyn DIDResolver {
        self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl DIDResolver for DIDPLC {
    async fn resolve(
        &self,
        did: &str,
        _input_metadata: &ResolutionInputMetadata,
    ) -> (
        ResolutionMetadata,
        Option<Document>,
        Option<DocumentMetadata>,
    ) {
        let res = match self
            .client
            .get(format!("{}/{}", self.host, did))
            .send()
            .await
        {
            Ok(res) => res,
            Err(err) => {
                return (
                    ResolutionMetadata::from_error(&format!("Failed to get URL: {:?}", err)),
                    None,
                    None,
                )
            }
        };

        match res.status().as_u16() {
            200 => {
                let text = match res.text().await {
                    Ok(json) => json,
                    Err(err) => {
                        return (
                            ResolutionMetadata::from_error(&format!(
                                "Failed to parse JSON response: {:?}",
                                err
                            )),
                            None,
                            None,
                        )
                    }
                };

                match Document::from_json(text.as_str()) {
                    Ok(document) => (
                        ResolutionMetadata::default(),
                        Some(document),
                        None,
                    ),
                    Err(err) => (
                        ResolutionMetadata::from_error(&format!(
                            "Unable to parse DID document: {:?}",
                            err
                        )),
                        None,
                        None,
                    ),
                }
            }
            404 => (
                ResolutionMetadata::from_error(&format!("DID not found: {}", did)),
                None,
                None,
            ),
            _ => (
                ResolutionMetadata::from_error(&format!(
                    "Failed to resolve DID: {}",
                    res.status()
                )),
                None,
                None,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_didplc_resolve() {
        let didplc = DIDPLC::default();
        let did = "did:plc:ui5pgpumwvufhfnnz52c4lyl";
        let (res_metadata, document, _) = didplc.resolve(did, &ResolutionInputMetadata::default()).await;

        assert!(res_metadata.error.is_none());
        assert!(document.is_some());
    }
}
