use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Digest, Sha256};
use base64::Engine;
use cid::Cid;

use crate::keypair::Keypair;

#[derive(Clone)]
pub enum PLCOperationType {
    Operation,
    Tombstone,
}

impl PLCOperationType {
    fn to_string(&self) -> &str {
        match self {
            Self::Operation => "plc_operation",
            Self::Tombstone => "plc_tombstone",
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        match s {
            "plc_operation" => Some(Self::Operation),
            "plc_tombstone" => Some(Self::Tombstone),
            "create" => Some(Self::Operation),
            _ => None,
        }
    }
}

impl Serialize for PLCOperationType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string())
    }
}

impl<'de> Deserialize<'de> for PLCOperationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_string(&s).ok_or(serde::de::Error::custom("Invalid PLCOperationType"))
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[serde(rename = "type")]
    type_: String,
    endpoint: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedPLCOperation {
    #[serde(rename = "type")]
    type_: PLCOperationType,
    rotation_keys: Vec<String>,
    verification_methods: HashMap<String, String>,
    also_known_as: Vec<String>,
    services: HashMap<String, Service>,
    prev: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignedPLCOperation {
    #[serde(flatten)]
    unsigned: UnsignedPLCOperation,
    sig: String,
}

impl UnsignedPLCOperation {
    pub fn to_json(&self) -> String {
        let value = serde_json::to_value(self).unwrap();
        match self.type_ {
            PLCOperationType::Operation => serde_json::to_string(&value).unwrap(),
            PLCOperationType::Tombstone => {
                let mut map = serde_json::Map::new();
                map.insert(
                    "type".to_string(),
                    serde_json::Value::String("plc_tombstone".to_string()),
                );
                map.insert(
                    "prev".to_string(),
                    serde_json::Value::String(self.prev.clone().unwrap()),
                );
                serde_json::to_string(&serde_json::Value::Object(map)).unwrap()
            }
        }
    }

    pub fn to_signed(&self, mut keypair: impl Keypair) -> Result<SignedPLCOperation, crate::Error> {
        let json = self.to_json();
        let dag = serde_ipld_dagcbor::to_vec(&json).unwrap();
        let sig = keypair.sign(&dag.as_slice())?;
        let engine = base64::engine::general_purpose::URL_SAFE;
        let sig = engine.encode(sig);
        Ok(SignedPLCOperation {
            unsigned: self.clone(),
            sig,
        })
    }
}

impl SignedPLCOperation {
    pub fn to_json(&self) -> String {
        let json_string = self.unsigned.to_json();
        let json_value: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        let mut json = json_value.as_object().unwrap().clone();
        json.insert(
            "sig".to_string(),
            serde_json::Value::String(self.sig.clone()),
        );
        serde_json::to_string(&json).unwrap()
    }

    pub fn to_cid(&self) -> String {
        let json = self.to_json();

        let result = Sha256::digest(serde_json::to_string(&json).unwrap().as_bytes());
        let cid = Cid::new_v1(
            0xb7112,
            cid::multihash::Multihash::<64>::from_bytes(result.as_slice()).unwrap(),
        );
        cid.to_string()
    }
}

fn assure_at_prefix(s: &str) -> String {
    if s.starts_with("at://") {
        s.to_string()
    } else {
        format!("at://{}", s)
    }
}

fn assure_http(s: &str) -> String {
    if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("https://{}", s)
    }
}

pub fn normalize_op(json: serde_json::Value) -> serde_json::Value {
    let json = json.as_object().unwrap().clone();
    let mut normalized_json = serde_json::Map::new();

    if json.get("type").unwrap() == "create" {
        // This is a legacy genesis operation format
        let mut rotation_keys: Vec<String> = vec![];
        let mut verification_methods: HashMap<String, String> = HashMap::new();
        let mut also_known_as: Vec<String> = vec![];
        let mut services: HashMap<String, Service> = HashMap::new();

        if !json.get("recoveryKey").unwrap().is_null() {
            rotation_keys.push(json.get("recoveryKey").unwrap().to_string());
        }
        if !json.get("signingKey").unwrap().is_null() {
            let key = json.get("signingKey").unwrap().to_string();
            rotation_keys.push(key.clone());
            verification_methods.insert("atproto".to_string(), key);
        }
        if !json.get("handle").unwrap().is_null() {
            also_known_as.push(assure_at_prefix(
                json.get("handle").unwrap().as_str().unwrap(),
            ));
        }
        if !json.get("service").unwrap().is_null() {
            services.insert(
                "atproto_pds".to_string(),
                Service {
                    type_: "AtprotoPersonalDataServer".to_string(),
                    endpoint: assure_http(
                        json.get("service")
                            .unwrap()
                            .get("endpoint")
                            .unwrap()
                            .as_str()
                            .unwrap(),
                    ),
                },
            );
        }
        normalized_json.insert(
            "type".to_string(),
            serde_json::Value::String("plc_operation".to_string()),
        );
        normalized_json.insert("prev".to_string(), serde_json::Value::Null);
        normalized_json.insert(
            "rotationKeys".to_string(),
            serde_json::Value::Array(Vec::from_iter(
                rotation_keys
                    .into_iter()
                    .map(|s| serde_json::Value::String(s)),
            )),
        );
        normalized_json.insert(
            "verificationMethods".to_string(),
            serde_json::to_value(verification_methods).unwrap(),
        );
        normalized_json.insert(
            "alsoKnownAs".to_string(),
            serde_json::Value::Array(Vec::from_iter(
                also_known_as
                    .into_iter()
                    .map(|s| serde_json::Value::String(s)),
            )),
        );
    } else {
        for (key, value) in json.iter() {
            normalized_json.insert(key.clone(), value.clone());
        }
    }

    if !normalized_json.get("alsoKnownAs").unwrap().is_null() {
        let mut also_known_as = vec![];
        for value in normalized_json
            .get("alsoKnownAs")
            .unwrap()
            .as_array()
            .unwrap()
        {
            also_known_as.push(assure_at_prefix(value.as_str().unwrap()));
        }
        normalized_json.insert(
            "alsoKnownAs".to_string(),
            serde_json::Value::Array(Vec::from_iter(
                also_known_as
                    .into_iter()
                    .map(|s| serde_json::Value::String(s)),
            )),
        );
    }

    serde_json::to_value(normalized_json).unwrap()
}