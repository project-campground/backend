extern crate thiserror;

use async_trait::async_trait;
use didkit::{
    DIDMethod, DIDResolver, Document, DocumentMetadata, ResolutionInputMetadata,
    ResolutionMetadata,
};
use operation::{PLCOperation, Service, SignedOperation, SignedPLCOperation, UnsignedPLCOperation};
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

        Ok(PLCOperation::UnsignedPLC(serde_json::from_str::<UnsignedPLCOperation>(&body)
            .map_err(|e| PLCError::Other(e.into()))?
        ))
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
    use operation::PLCOperationType;

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
    async fn test_didplc_get_log() {
        let didplc = DIDPLC::default();
        let did = "did:plc:ui5pgpumwvufhfnnz52c4lyl";
        let log = didplc.get_log(did).await;

        assert!(log.is_ok());
        assert!(log.unwrap().len() > 0);
    }

    #[actix_rt::test]
    async fn test_didplc_get_audit_log() {
        let didplc = DIDPLC::default();
        let did = "did:plc:ui5pgpumwvufhfnnz52c4lyl";
        let log = didplc.get_audit_log(did).await;

        assert!(log.is_ok());
        assert!(log.unwrap().len() > 0);
    }

    #[actix_rt::test]
    async fn test_didplc_get_last_log() {
        let didplc = DIDPLC::default();
        let did = "did:plc:ui5pgpumwvufhfnnz52c4lyl";
        let log = didplc.get_last_log(did).await;

        assert!(log.is_ok());
    }

    #[actix_rt::test]
    async fn test_didplc_get_current_state() {
        let didplc = DIDPLC::default();
        let did = "did:plc:ui5pgpumwvufhfnnz52c4lyl";
        let log = didplc.get_current_state(did).await;

        assert!(log.is_ok());
    }

    #[actix_rt::test]
    async fn test_didplc_operations() {
        let didplc = DIDPLC::new(PLC_HOST);
        let recovery_key = Keypair::generate(BlessedAlgorithm::K256);
        let signing_key = Keypair::generate(BlessedAlgorithm::K256);
        let verification_key = Keypair::generate(BlessedAlgorithm::K256);

        let create_op = OperationBuilder::new(&didplc)
            .with_key(&signing_key)
            .with_validation_key(&verification_key)
            .add_rotation_key(&recovery_key)
            .add_rotation_key(&signing_key)
            .with_handle("example.test".to_owned())
            .with_pds("example.test".to_owned())
            .build(PLCOperationType::Operation)
            .await;

        assert!(create_op.is_ok(), "Failed to build create op: {:?}", create_op.err());
        let create_op = create_op.unwrap();
        let did = &create_op.to_did().expect("Failed to turn op to DID");

        let create_res = didplc.execute_op(did, &create_op).await;

        assert!(create_res.is_ok(), "Failed to execute create op: {:?}", create_res.err());
        let create_res = create_res.unwrap();

        assert!(create_res.status == 200, "Failed to execute create op: status = {}, body = {:?}", create_res.status, create_res.body);
        assert!(&create_res.did == did, "Failed to execute create op: did = {}, expected = {}", create_res.did, did);

        let update_op = OperationBuilder::for_did(&didplc, did.clone())
            .with_key(&signing_key)
            .with_validation_key(&verification_key)
            .add_rotation_key(&recovery_key)
            .add_rotation_key(&signing_key)
            .with_handle("touma.example.test".to_owned())
            .with_pds("example.test".to_owned())
            .build(PLCOperationType::Operation)
            .await;

        assert!(update_op.is_ok(), "Failed to build update op: {:?}", update_op.err());
        let update_op = update_op.unwrap();
        let update_res = didplc.execute_op(did, &update_op).await;
        assert!(update_res.is_ok(), "Failed to execute update op: {:?}", update_res.err());

        let update_res = update_res.unwrap();
        assert!(update_res.status == 200, "Failed to execute update op: status = {}, body = {:?}, json = {}", update_res.status, update_res.body, update_op.to_json());
        assert!(&update_res.did == did, "Failed to execute update op: did = {}, expected = {}", update_res.did, did);

        let deactivate_op = OperationBuilder::for_did(&didplc, did.clone())
            .with_key(&signing_key)
            .with_validation_key(&verification_key)
            .add_rotation_key(&recovery_key)
            .add_rotation_key(&signing_key)
            .with_handle("touma.example.test".to_owned())
            .with_pds("example.test".to_owned())
            .build(PLCOperationType::Tombstone)
            .await;
        assert!(deactivate_op.is_ok(), "Failed to build deactivate op: {:?}", deactivate_op.err());
        let deactivate_op = deactivate_op.unwrap();
        let deactivate_res = didplc.execute_op(did, &deactivate_op).await;
        assert!(deactivate_res.is_ok(), "Failed to execute deactivate op: {:?}, json = {}", deactivate_res.err(), deactivate_op.to_json());

        let deactivate_res = deactivate_res.unwrap();
        assert!(deactivate_res.status == 200, "Failed to execute deactivate op: status = {}, body = {:?}", deactivate_res.status, deactivate_res.body);
        assert!(&deactivate_res.did == did, "Failed to execute deactivate op: did = {}, expected = {}", deactivate_res.did, did);

        let recover_op = OperationBuilder::for_did(&didplc, did.clone())
            .with_key(&recovery_key)
            .with_validation_key(&verification_key)
            .add_rotation_key(&recovery_key)
            .add_rotation_key(&signing_key)
            .with_handle("touma.example.test".to_owned())
            .with_pds("example.test".to_owned())
            .build(PLCOperationType::Operation)
            .await;
        assert!(recover_op.is_ok(), "Failed to build recover op: {:?}", recover_op.err());
        let recover_op = recover_op.unwrap();
        let recover_res = didplc.execute_op(did, &recover_op).await;
        assert!(recover_res.is_ok(), "Failed to execute recover op: {:?}, json = {}", recover_res.err(), recover_op.to_json());

        let recover_res = recover_res.unwrap();
        assert!(recover_res.status == 200, "Failed to execute recover op: status = {}, body = {:?}", recover_res.status, recover_res.body);
        assert!(&recover_res.did == did, "Failed to execute recover op: did = {}, expected = {}", recover_res.did, did);
    }
}
