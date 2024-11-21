use std::collections::BTreeMap;
use anyhow::Result;
use rsky_pds::common::ipld::cid_for_cbor;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Service {
    #[serde(rename = "type")]
    pub r#type: String,
    pub endpoint: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocumentData {
    pub did: String,
    #[serde(rename = "rotationKeys")]
    pub rotation_keys: Vec<String>,
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateOpV1 {
    #[serde(rename = "type")]
    pub r#type: String, // string literal `create`
    #[serde(rename = "signingKey")]
    pub signing_key: String,
    #[serde(rename = "recoveryKey")]
    pub recovery_key: String,
    pub handle: String,
    pub service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Operation {
    #[serde(rename = "type")]
    pub r#type: String, // string literal `plc_operation`
    #[serde(rename = "rotationKeys")]
    pub rotation_keys: Vec<String>,
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, Service>,
    // Omit<t.UnsignedOperation, 'prev'>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

impl Operation {
    pub fn include_prev(self) -> OpIncludePrev {
        OpIncludePrev {
            r#type: self.r#type.clone(),
            rotation_keys: self.rotation_keys.clone(),
            verification_methods: self.verification_methods.clone(),
            also_known_as: self.also_known_as.clone(),
            services: self.services.clone(),
            prev: self.prev.clone(),
            sig: self.sig.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpIncludePrev {
    #[serde(rename = "type")]
    pub r#type: String, // string literal `plc_operation`
    #[serde(rename = "rotationKeys")]
    pub rotation_keys: Vec<String>,
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, Service>,
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tombstone {
    #[serde(rename = "type")]
    pub r#type: String, // string literal `plc_tombstone`
    pub prev: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)] // Needs to be signed, so we don't want an additional tag
pub enum CompatibleOpOrTombstone {
    CreateOpV1(CreateOpV1),
    Operation(Operation),
    Tombstone(Tombstone),
}

impl CompatibleOpOrTombstone {
    pub fn set_sig(&mut self, sig: String) -> () {
        match self {
            Self::CreateOpV1(create) => create.sig = Some(sig),
            Self::Operation(op) => op.sig = Some(sig),
            Self::Tombstone(tombstone) => tombstone.sig = Some(sig),
        }
    }

    pub fn get_sig(&mut self) -> &Option<String> {
        match self {
            Self::CreateOpV1(create) => &create.sig,
            Self::Operation(op) => &op.sig,
            Self::Tombstone(tombstone) => &tombstone.sig,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum CompatibleOp {
    CreateOpV1(CreateOpV1),
    Operation(Operation),
}

impl CompatibleOp {
    pub fn to_cid(&self) -> Result<libipld::Cid> {
        match self {
            Self::CreateOpV1(op) => Ok(cid_for_cbor(op)?),
            Self::Operation(op) => Ok(cid_for_cbor(&op.clone().include_prev())?),
        }
    }
    
    pub fn set_sig(&mut self, sig: String) -> () {
        match self {
            Self::CreateOpV1(create) => create.sig = Some(sig),
            Self::Operation(op) => op.sig = Some(sig),
        }
    }

    pub fn get_sig(&mut self) -> &Option<String> {
        match self {
            Self::CreateOpV1(create) => &create.sig,
            Self::Operation(op) => &op.sig,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)] // will be posted to API so needs to not be tagged
pub enum OpOrTombstone {
    Operation(Operation),
    Tombstone(Tombstone),
}

impl OpOrTombstone {
    pub fn set_sig(&mut self, sig: String) -> () {
        match self {
            Self::Operation(op) => op.sig = Some(sig),
            Self::Tombstone(tombstone) => tombstone.sig = Some(sig),
        }
    }

    pub fn get_sig(&mut self) -> &Option<String> {
        match self {
            Self::Operation(op) => &op.sig,
            Self::Tombstone(tombstone) => &tombstone.sig,
        }
    }
}
