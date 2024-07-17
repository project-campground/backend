extern crate thiserror;

use std::collections::HashMap;

use didkit::{
    DIDMethod, DIDResolver, Document, DocumentMetadata, ResolutionInputMetadata, ResolutionMetadata,
    ssi::did::{DIDMethodTransaction, DIDMethodError}
};
use async_trait::async_trait;
use operation::{PLCOperation, PLCOperationType, Service, SignedOperation, UnsignedOperation, UnsignedPLCOperation};
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

#[derive(Debug, thiserror::Error)]
pub enum PLCError {
    #[error("Failed to create PLC: {0}")]
    Create(u16),

    #[error("Failed to deactivate PLC: {0}")]
    Deactivated(String),

    #[error("Failed to update PLC: {0}")]
    Update(String),
    
    #[error("Failed to recover PLC: {0}")]
    Recover(String),

    #[error("Misordered operation")]
    MisorderedOperation,

    #[error("Recovery too late")]
    LateRecovery,

    #[error("Signature is invalid")]
    InvalidSignature,
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

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl DIDMethod for DIDPLC {
    fn name(&self) -> &'static str {
        "did:plc"
    }

    fn to_resolver(&self) -> &dyn DIDResolver {
        self
    }

    async fn submit_transaction(&self, _tx: DIDMethodTransaction) -> Result<serde_json::Value, DIDMethodError> {
        match _tx.did_method.as_str() {
            "create" => {
                let map: HashMap<String, serde_json::Value> = match serde_json::from_value(_tx.value) {
                    Ok(map) => map,
                    Err(e) => return Err(DIDMethodError::Other(e.into())),
                };
                let key = map.get("key").unwrap().as_str().unwrap();
                let op: UnsignedPLCOperation = match serde_json::from_value(map.get("operation").unwrap().clone()) {
                    Ok(op) => op,
                    Err(e) => return Err(DIDMethodError::Other(e.into())),
                };
                let op = match op.to_signed(key) {
                    Ok(op) => op,
                    Err(e) => return Err(DIDMethodError::Other(e.into())),
                };
                let did = match op.to_did() {
                    Ok(did) => did,
                    Err(e) => return Err(DIDMethodError::Other(e.into())),
                };

                match self
                    .client
                    .post(format!("{}/{}", self.host, did))
                    .body(op.to_json())
                    .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"))
                    .send()
                    .await {
                        Ok(res) => {
                            if res.status().is_success() {
                                match res.json().await {
                                    Ok(json) => Ok(json),
                                    Err(e) => Err(DIDMethodError::Other(e.into())),
                                }
                            } else {
                                let status = res.status();
                                let text = &res.text().await;
                                match text {
                                    Ok(text) => println!("{}", text),
                                    Err(_) => {}
                                };
                                Err(DIDMethodError::Other(PLCError::Create(status.as_u16()).into()))
                            }
                        },
                        Err(e) => Err(DIDMethodError::Other(e.into())),
                    }
            }
            "update" => {
                let map: HashMap<String, serde_json::Value> = match serde_json::from_value(_tx.value) {
                    Ok(map) => map,
                    Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                };
                let did = map.get("did").unwrap().as_str().unwrap();
                let state = match self.get_current_state(did).await {
                    Ok(state) => state,
                    Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                };
                match state {
                    PLCOperation::SignedPLC(op) => {
                        let mut unsigned = op.unsigned.clone();
                        if let Some(rotation_keys) = map.get("rotationKeys") {
                            let rotation_keys: Vec<String> = match serde_json::from_value(rotation_keys.clone()) {
                                Ok(keys) => keys,
                                Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                            };
                            unsigned.rotation_keys = rotation_keys;
                        }
                        if let Some(verification_methods) = map.get("verificationMethods") {
                            let verification_methods: HashMap<String, String> = match serde_json::from_value(verification_methods.clone()) {
                                Ok(map) => map,
                                Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                            };
                            unsigned.verification_methods = verification_methods;
                        }
                        if let Some(also_known_as) = map.get("alsoKnownAs") {
                            let also_known_as: Vec<String> = match serde_json::from_value(also_known_as.clone()) {
                                Ok(list) => list,
                                Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                            };
                            unsigned.also_known_as = also_known_as;
                        }
                        if let Some(services) = map.get("services") {
                            let services: HashMap<String, Service> = match serde_json::from_value(services.clone()) {
                                Ok(map) => map,
                                Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                            };
                            unsigned.services = services;
                        }
                        unsigned.prev = match map.get("prev") {
                            Some(prev) => match prev.as_str() {
                                Some(_) => match op.to_cid() {
                                    Ok(cid) => Some(cid),
                                    Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                                },
                                None => None,
                            },
                            None => None,
                        };
                        unsigned.type_ = PLCOperationType::Operation;
                        let signed = match unsigned.to_signed(did) {
                            Ok(signed) => signed,
                            Err(e) => return Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                        };

                        match self
                            .client
                            .post(format!("{}/{}", self.host, did))
                            .body(signed.to_json())
                            .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"))
                            .send()
                            .await {
                                Ok(res) => {
                                    if res.status().is_success() {
                                        match res.json().await {
                                            Ok(json) => Ok(json),
                                            Err(e) => Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                                        }
                                    } else {
                                        Err(DIDMethodError::Other(PLCError::Create(res.status().as_u16()).into()))
                                    }
                                },
                                Err(e) => Err(DIDMethodError::Other(PLCError::Update(e.to_string()).into())),
                            }
                    },
                    _ => return Err(DIDMethodError::Other(Error::UnsignedOperation.into())),
                }
            }
            "deactivate" => {
                let map: HashMap<String, serde_json::Value> = match serde_json::from_value(_tx.value) {
                    Ok(map) => map,
                    Err(e) => return Err(DIDMethodError::Other(e.into())),
                };
                let did = map.get("did").unwrap().as_str().unwrap();
                let key = map.get("key").unwrap().as_str().unwrap();
                let audit_log = match self.get_audit_log(did).await {
                    Ok(log) => log,
                    Err(e) => return Err(DIDMethodError::Other(e.into())),
                };
                let log = audit_log.last().unwrap();
                if let PLCOperation::SignedPLC(op) = &log.operation {
                    if op.unsigned.type_.to_string() == PLCOperationType::Tombstone.to_string() {
                        return Err(DIDMethodError::Other(PLCError::Deactivated("DID already deactivated".to_string()).into()));
                    }
                    if let Ok(cid) = op.to_cid() {
                        let mut new_op = op.unsigned.clone();
                        new_op.prev = Some(cid);
                        new_op.type_ = PLCOperationType::Tombstone;

                        let _ = match new_op.to_signed(&key) {
                            Ok(signed) => {
                                let signed = signed.to_json();

                                match self
                                    .client
                                    .post(format!("{}/{}", self.host, did))
                                    .body(signed)
                                    .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"))
                                    .send()
                                    .await {
                                        Ok(res) => {
                                            if res.status().is_success() {
                                                match res.json().await {
                                                    Ok(json) => Ok(json),
                                                    Err(e) => Err(DIDMethodError::Other(e.into())),
                                                }
                                            } else {
                                                Err(DIDMethodError::Other(PLCError::Create(res.status().as_u16()).into()))
                                            }
                                        },
                                        Err(e) => Err(DIDMethodError::Other(e.into())),
                                    }?
                            },
                            Err(e) => return Err(DIDMethodError::Other(e.into())),
                        };
                    }
                }
                Err(DIDMethodError::Other(PLCError::Deactivated("DID not found".to_string()).into()))
            }
            "recover" => {
                let map: HashMap<String, serde_json::Value> = match serde_json::from_value(_tx.value) {
                    Ok(map) => map,
                    Err(e) => return Err(DIDMethodError::Other(PLCError::Recover(e.to_string()).into())),
                };
                let did = map.get("did").unwrap().as_str().unwrap();
                let key = map.get("key").unwrap().as_str().unwrap();
                let audit_log = match self.get_audit_log(did).await {
                    Ok(log) => log,
                    Err(e) => return Err(DIDMethodError::Other(PLCError::Recover(e.to_string()).into())),
                };
                let log = audit_log.last().unwrap();
                if let PLCOperation::SignedPLC(op) = &log.operation {
                    if op.unsigned.type_.to_string() == PLCOperationType::Tombstone.to_string() {
                        let mut new_op = op.unsigned.clone();
                        new_op.type_ = PLCOperationType::Operation;

                        let _ = match new_op.to_signed(&key) {
                            Ok(signed) => {
                                let signed = signed.to_json();

                                match self
                                    .client
                                    .post(format!("{}/{}", self.host, did))
                                    .body(signed)
                                    .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"))
                                    .send()
                                    .await {
                                        Ok(res) => {
                                            if res.status().is_success() {
                                                match res.json().await {
                                                    Ok(json) => Ok(json),
                                                    Err(e) => Err(DIDMethodError::Other(PLCError::Recover(e.to_string()).into())),
                                                }
                                            } else {
                                                Err(DIDMethodError::Other(PLCError::Recover(res.text().await.unwrap()).into()))
                                            }
                                        },
                                        Err(e) => Err(DIDMethodError::Other(PLCError::Recover(e.to_string()).into())),
                                    }?
                            },
                            Err(e) => return Err(DIDMethodError::Other(PLCError::Recover(e.to_string()).into())),
                        };
                    } else {
                        return Err(DIDMethodError::Other(PLCError::Recover("DID not deactivated".to_string()).into()));
                    }
                }
                Err(DIDMethodError::Other(PLCError::Recover("DID not found".to_string()).into()))
            }
            _ => Err(DIDMethodError::NotImplemented("Unknown method"))
        }
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
            rotation_keys: rotation_keys.clone().into_iter().map(|v| v.to_did_key().unwrap()).collect(),
            verification_methods: {
                let mut map = HashMap::new();
                map.insert("atproto".to_string(), validation_key.to_did_key().unwrap());
                map
            },
            also_known_as: vec![assure_at_prefix(handle)],
            services: {
                let mut map = HashMap::new();
                map.insert("atproto_pds".to_string(), Service {
                    type_: "AtprotoPersonalDataServer".to_string(),
                    endpoint: assure_http(service),
                });
                map
            },
            prev: None,
        };

        Ok(DIDMethodTransaction {
            did_method: "create".to_string(),
            value: serde_json::to_value({
                let mut map = HashMap::new();
                map.insert("operation", serde_json::to_value(op).unwrap());
                map.insert("key", serde_json::to_value(
                    rotation_keys
                        .last()
                        .unwrap()
                        .to_private_key()
                        .unwrap()
                        .as_str()
                    ).unwrap());
                map
            }).unwrap(),
        })
    }

    fn update(&self, _update: didkit::DIDUpdate) -> Result<DIDMethodTransaction, DIDMethodError> {
        // Handle most of this logic in submit_transaction because reqwest requires async
        let value = serde_json::to_value(&_update.options).unwrap();
        let mut value: HashMap<String, serde_json::Value> = serde_json::from_value(value).unwrap();
        let did = value.get("did").unwrap().as_str().unwrap();
        value.insert("did".to_string(), serde_json::Value::String(did.to_string()));
        Ok(DIDMethodTransaction {
            did_method: "update".to_string(),
            value: serde_json::to_value(value).unwrap(),
        })
    }

    fn deactivate(&self, _deactivate: didkit::DIDDeactivate) -> Result<DIDMethodTransaction, DIDMethodError> {
        let value = serde_json::to_value(&_deactivate.options).unwrap();
        let mut value: HashMap<String, serde_json::Value> = serde_json::from_value(value).unwrap();
        let did = value.get("did").unwrap().as_str().unwrap();
        value.insert("did".to_string(), serde_json::Value::String(did.to_string()));
        Ok(DIDMethodTransaction {
            did_method: "deactivate".to_string(),
            value: serde_json::to_value(value).unwrap(),
        })
    }

    fn recover(&self, _recover: didkit::DIDRecover) -> Result<DIDMethodTransaction, DIDMethodError> {
        let value = serde_json::to_value(&_recover.options).unwrap();
        let mut value: HashMap<String, serde_json::Value> = serde_json::from_value(value).unwrap();
        let did = value.get("did").unwrap().as_str().unwrap();
        value.insert("did".to_string(), serde_json::Value::String(did.to_string()));
        Ok(DIDMethodTransaction {
            did_method: "recover".to_string(),
            value: serde_json::to_value(value).unwrap(),
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
    use std::collections::BTreeMap;

    use super::*;

    const PLC_HOST: &str = "https://plc.directory"; // "http://localhost:2894";

    #[actix_rt::test]
    async fn test_didplc_resolve() {
        let didplc = DIDPLC::default();
        let did = "did:plc:ui5pgpumwvufhfnnz52c4lyl";
        let (res_metadata, document, _) = didplc.resolve(did, &ResolutionInputMetadata::default()).await;

        assert!(res_metadata.error.is_none());
        assert!(document.is_some());
    }

    #[actix_rt::test]
    async fn test_didplc_create() {
        let didplc = DIDPLC::new(PLC_HOST);
        let recovery_key = Keypair::generate(BlessedAlgorithm::P256);
        let signing_key = Keypair::generate(BlessedAlgorithm::P256);
        let verification_key = Keypair::generate(BlessedAlgorithm::P256);
        let create = didkit::DIDCreate {
            options: {
                let mut map = BTreeMap::new();
                map.insert("rotationKeys".to_string(), serde_json::to_value(vec![recovery_key, signing_key]).unwrap());
                map.insert("validationKey".to_string(), serde_json::to_value(verification_key).unwrap());
                map.insert("handle".to_string(), serde_json::Value::String("example.test".to_string()));
                map.insert("service".to_string(), serde_json::Value::String("https://example.test".to_string()));
                map
            },
            update_key: None,
            recovery_key: None,
            verification_key: None,
        };
        let tx = didplc.create(create);
        assert!(tx.is_ok());

        let result = didplc.submit_transaction(tx.unwrap()).await;
        match &result {
            Err(err) => println!("Error: {:?}", err),
            Ok(_) => (),
        }
        assert!(result.is_ok());
    }
}
