extern crate thiserror;

use async_trait::async_trait;
use didkit::{
    DIDMethod, DIDResolver, Document, DocumentMetadata, ResolutionInputMetadata,
    ResolutionMetadata,
};
use operation::{PLCOperation, Service, SignedOperation, SignedPLCOperation};
use util::op_from_json;

mod audit;
mod error;
mod keypair;
mod multicodec;
mod op_builder;
pub mod operation;
mod util;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
pub const DEFAULT_HOST: &str = "https://plc.directory";

pub use audit::{AuditLog, DIDAuditLogs};
pub use error::PLCError;
pub use keypair::{BlessedAlgorithm, Keypair};
pub use op_builder::OperationBuilder;

pub struct PLCOperationResult {
    pub did: String,
    pub status: u16,
    pub body: String,
}

/// did:plc Method
///
/// [Specification](https://web.plc.directory/spec/v0.1/did-plc)
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

    pub async fn execute_op(&self, did: &str, op: &SignedPLCOperation) -> Result<PLCOperationResult, PLCError> {
        let res = self
            .client
            .post(format!("{}/{}", self.host, did))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(op.to_json())
            .send()
            .await?;

        let status = res.status().as_u16();
        let body: String = res.text().await?;
        Ok(PLCOperationResult {
            did: did.to_string(),
            status: status,
            body,
        })
    }

    pub async fn get_log(&self, did: &str) -> Result<Vec<PLCOperation>, PLCError> {
        let res = self
            .client
            .get(format!("{}/{}/log", self.host, did))
            .send()
            .await?;

        let body: String = res.text().await?;
        let mut operations: Vec<PLCOperation> = vec![];
        let json: Vec<serde_json::Value> =
            serde_json::from_str(&body).map_err(|e| PLCError::Other(e.into()))?;

        for op in json {
            operations.push(
                op_from_json(
                    serde_json::to_string(&op)
                        .map_err(|e| PLCError::Other(e.into()))?
                        .as_str(),
                )
                .map_err(|e| PLCError::Other(e.into()))?,
            );
        }

        Ok(operations)
    }

    pub async fn get_audit_log(&self, did: &str) -> Result<DIDAuditLogs, PLCError> {
        let res = self
            .client
            .get(format!("{}/{}/log/audit", self.host, did))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(PLCError::Http(
                res.status().as_u16(),
                res.text().await.unwrap_or_default(),
            ));
        }

        let body: String = res.text().await?;

        Ok(DIDAuditLogs::from_json(&body).map_err(|e| PLCError::Other(e.into()))?)
    }

    pub async fn get_last_log(&self, did: &str) -> Result<PLCOperation, PLCError> {
        let res = self
            .client
            .get(format!("{}/{}/log/last", self.host, did))
            .send()
            .await?;

        let body: String = res.text().await?;
        let op: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| PLCError::Other(e.into()))?;

        Ok(op_from_json(
            serde_json::to_string(&op)
                .map_err(|e| PLCError::Other(e.into()))?
                .as_str(),
        )?)
    }

    pub async fn get_current_state(&self, did: &str) -> Result<PLCOperation, PLCError> {
        let res = self
            .client
            .get(format!("{}/{}/data", self.host, did))
            .send()
            .await?;

        let body: String = res.text().await?;
        let op: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| PLCError::Other(e.into()))?;

        Ok(op_from_json(
            serde_json::to_string(&op)
                .map_err(|e| PLCError::Other(e.into()))?
                .as_str(),
        )?)
    }
}

impl Default for DIDPLC {
    fn default() -> Self {
        Self::new(DEFAULT_HOST)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
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
                    Ok(document) => (ResolutionMetadata::default(), Some(document), None),
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
                ResolutionMetadata::from_error(&format!("Failed to resolve DID: {}", res.status())),
                None,
                None,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    const PLC_HOST: &str = "https://plc.directory"; // "http://localhost:2894";

    #[actix_rt::test]
    async fn test_didplc_resolve() {
        let didplc = DIDPLC::default();
        let did = "did:plc:ui5pgpumwvufhfnnz52c4lyl";
        let (res_metadata, document, _) = didplc
            .resolve(did, &ResolutionInputMetadata::default())
            .await;

        assert!(res_metadata.error.is_none());
        assert!(document.is_some());
    }

    #[actix_rt::test]
    async fn test_didplc_operations() {
        let didplc = DIDPLC::new(PLC_HOST);
        let recovery_key = Keypair::generate(BlessedAlgorithm::P256);
        let signing_key = Keypair::generate(BlessedAlgorithm::P256);
        let verification_key = Keypair::generate(BlessedAlgorithm::P256);
        // TODO: Rewrite this test for the new operation builder
    }
}
