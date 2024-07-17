extern crate thiserror;

use std::collections::HashMap;

use didkit::{
    DIDMethod, DIDResolver, Document, DocumentMetadata, ResolutionInputMetadata, ResolutionMetadata,
    ssi::did::{DIDMethodTransaction, DIDMethodError}
};
use async_trait::async_trait;
use operation::{PLCOperation, PLCOperationType, Service, UnsignedPLCOperation};
use util::{assure_at_prefix, assure_http, op_from_json};

pub mod operation;
pub mod keypair;
mod multicodec;
pub mod audit;
mod util;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
pub const DEFAULT_HOST: &str = "https://plc.directory";

pub use keypair::{Keypair, BlessedAlgorithm};
pub use audit::{AuditLog, DIDAuditLogs};

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

    #[error("Invalid operation type: {0}")]
    InvalidOperationType(String),

    #[error("ECDSA elliptic curve error: {0}")]
    ECDSAEllipticCurve(#[from] ecdsa::elliptic_curve::Error),
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

    pub async fn get_log(&self, did: &str) -> Result<Vec<PLCOperation>, Error> {
        let res = self
            .client
            .get(format!("{}/{}/log", self.host, did))
            .send()
            .await?;

        let body: String = res.text().await?;
        let mut operations: Vec<PLCOperation> = vec![];
        let json: Vec<serde_json::Value> = serde_json::from_str(&body)?;

        for op in json {
            operations.push(op_from_json(serde_json::to_string(&op)?.as_str())?);
        }

        Ok(operations)
    }

    pub async fn get_audit_log(&self, did: &str) -> Result<DIDAuditLogs, Error> {
        let res = self
            .client
            .get(format!("{}/{}/log/audit", self.host, did))
            .send()
            .await?;

        let body: String = res.text().await?;

        Ok(DIDAuditLogs::from_json(&body)?)
    }

    pub async fn get_last_log(&self, did: &str) -> Result<PLCOperation, Error> {
        let res = self
            .client
            .get(format!("{}/{}/log/last", self.host, did))
            .send()
            .await?;

        let body: String = res.text().await?;
        let op: serde_json::Value = serde_json::from_str(&body)?;

        Ok(op_from_json(serde_json::to_string(&op)?.as_str())?)
    }

    pub async fn get_current_state(&self, did: &str) -> Result<PLCOperation, Error> {
        let res = self
            .client
            .get(format!("{}/{}/data", self.host, did))
            .send()
            .await?;

        let body: String = res.text().await?;
        let op: serde_json::Value = serde_json::from_str(&body)?;

        Ok(op_from_json(serde_json::to_string(&op)?.as_str())?)
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

    fn create(&self, _create: didkit::DIDCreate) -> Result<DIDMethodTransaction, DIDMethodError> {
        let rotation_keys = _create.options.get("rotationKeys").unwrap();
        let validation_key = _create.options.get("validationKey").unwrap();
        let handle = _create.options.get("handle").unwrap();
        let service = _create.options.get("service").unwrap();

        let rotation_keys: Vec<Keypair> = rotation_keys.as_array().unwrap().into_iter().map(|v| Keypair::from_value(v.clone()).unwrap()).collect();
        let validation_key = Keypair::from_value(validation_key.clone()).unwrap();
        let handle = handle.as_str().unwrap();
        let service = service.as_str().unwrap();

        let op = UnsignedPLCOperation {
            type_: PLCOperationType::Operation,
            rotation_keys: rotation_keys.into_iter().map(|v| v.to_did_key().unwrap()).collect(),
            verification_methods: {
                let mut map = HashMap::new();
                map.insert("atproto".to_string(), validation_key.to_did_key().unwrap());
                map
            },
            also_known_as: vec![assure_at_prefix(handle)],
            services: {
                let mut map = HashMap::new();
                map.insert("atproto_pds".to_string(), Service {
                    type_: "AtprotoData".to_string(),
                    endpoint: assure_http(service),
                });
                map
            },
            prev: None,
        };

        Ok(DIDMethodTransaction {
            did_method: "create".to_string(),
            value: serde_json::to_value(&op).unwrap(),
        })
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
