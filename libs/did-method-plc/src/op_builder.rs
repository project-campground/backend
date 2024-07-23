use std::collections::HashMap;

use crate::{
    operation::{PLCOperationType, SignedPLCOperation, UnsignedOperation, UnsignedPLCOperation},
    util::{assure_at_prefix, assure_http},
    Keypair, PLCError, Service, DIDPLC,
};

pub struct OperationBuilder<'a, 'k> {
    plc: &'a DIDPLC,
    key: Option<&'k Keypair>,
    did: Option<String>,
    rotation_keys: Vec<String>,
    services: HashMap<String, Service>,
    also_known_as: Vec<String>,
    verification_methods: HashMap<String, String>,
    prev: Option<String>,
}

impl<'a, 'k> OperationBuilder<'a, 'k> {
    pub fn new(plc: &'a DIDPLC) -> Self {
        OperationBuilder {
            plc,
            key: None,
            did: None,
            rotation_keys: vec![],
            services: HashMap::new(),
            also_known_as: vec![],
            verification_methods: HashMap::new(),
            prev: None,
        }
    }

    pub fn for_did(plc: &'a DIDPLC, did: String) -> Self {
        OperationBuilder {
            plc,
            key: None,
            did: Some(did),
            rotation_keys: vec![],
            services: HashMap::new(),
            also_known_as: vec![],
            verification_methods: HashMap::new(),
            prev: None,
        }
    }

    pub fn with_key(&mut self, key: &'k Keypair) -> &mut Self {
        self.key = Some(key);
        self
    }

    pub fn with_validation_key(&mut self, key: &Keypair) -> &mut Self {
        self.verification_methods
            .insert("atproto".to_string(), key.to_did_key().unwrap());
        self
    }

    pub fn with_handle(&mut self, handle: String) -> &mut Self {
        self.also_known_as.push(assure_at_prefix(&handle));
        self
    }

    pub fn with_pds(&mut self, pds: String) -> &mut Self {
        self.services.insert(
            "atproto_pds".to_string(),
            Service {
                type_: "AtprotoPersonalDataServer".to_string(),
                endpoint: assure_http(&pds),
            },
        );
        self
    }

    pub fn add_rotation_key(&mut self, key: &Keypair) -> &mut Self {
        self.rotation_keys.push(key.to_did_key().unwrap());
        self
    }

    pub fn add_known_as(&mut self, name: String) -> &mut Self {
        self.also_known_as.push(name);
        self
    }

    pub fn set_prev(&mut self, prev: String) -> &mut Self {
        self.prev = Some(prev);
        self
    }

    pub async fn build(&mut self, op_type: PLCOperationType) -> Result<SignedPLCOperation, PLCError> {
        if self.services.get("atproto_pds").is_none() {
            return Err(PLCError::InvalidOperation)
        }
        if self.key.is_none() {
            return Err(PLCError::InvalidOperation)
        }
        if self.rotation_keys.len() < 2 {
            return Err(PLCError::InvalidOperation)
        }
        if self.also_known_as.len() < 1 {
            return Err(PLCError::InvalidOperation)
        }
        if self.verification_methods.get("atproto").is_none() {
            return Err(PLCError::InvalidOperation)
        }
        if self.did.is_some() {
            // Not a genesis op
            match &self.prev {
                Some(_) => (),
                None => {
                    // Try and automatically retreive previous log CID
                    let audit_log = self.plc.get_audit_log(&self.did.as_ref().unwrap()).await?;
                    self.set_prev(audit_log.get_latest()?);
                    ()
                }
            }
        }
        let op = UnsignedPLCOperation {
            type_: op_type,
            verification_methods: self.verification_methods.clone(),
            services: self.services.clone(),
            rotation_keys: self.rotation_keys.clone(),
            also_known_as: self.also_known_as.clone(),
            prev: self.prev.clone(),
        };
        let key = &self
            .key
            .clone()
            .unwrap()
            .to_private_key()
            .map_err(|e| PLCError::Other(e.into()))?;
        op.to_signed(key.as_str()).map_err(|e| PLCError::Other(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLC_HOST: &str = "https://plc.directory";

    #[actix_rt::test]
    async fn test_operation_builder() {
        let plc = DIDPLC::new(PLC_HOST);
        let signing_key = Keypair::generate(crate::BlessedAlgorithm::P256);
        let recovery_key = Keypair::generate(crate::BlessedAlgorithm::P256);
        let mut builder = OperationBuilder::new(&plc);
        builder.with_key(&signing_key)
            .with_validation_key(&Keypair::generate(crate::BlessedAlgorithm::P256))
            .with_handle("example.test".to_string())
            .with_pds("https://example.test".to_string())
            .add_rotation_key(&recovery_key)
            .add_rotation_key(&signing_key);
        let op = builder.build(PLCOperationType::Operation).await;
        assert!(op.is_ok(), "Operation should build");
    }
}
