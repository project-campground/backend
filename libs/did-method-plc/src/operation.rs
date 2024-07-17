use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Serialize, Serializer};
use crate::util::op_from_json;
use crate::util::normalize_op;
use std::collections::HashMap;
use sha2::{Digest, Sha256};
use base32::Alphabet;
use base64::Engine;
use crate::Keypair;
use cid::Cid;

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

pub trait UnsignedOperation {
    fn to_json(&self) -> String;
    fn to_signed(&self, key: &str) -> Result<impl SignedOperation, crate::Error>;
}

pub trait SignedOperation {
    fn to_json(&self) -> String;
    fn to_cid(&self) -> Result<String, crate::Error>;
    fn to_did(&self) -> Result<String, crate::Error>;
    fn verify_sig(&self) -> Result<bool, crate::Error>;
}

pub enum PLCOperation {
    UnsignedGenesis(UnsignedGenesisOperation),
    SignedGenesis(SignedGenesisOperation),
    UnsignedPLC(UnsignedPLCOperation),
    SignedPLC(SignedPLCOperation),
}

impl Serialize for PLCOperation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match self {
            Self::UnsignedGenesis(op) => op.serialize(serializer),
            Self::SignedGenesis(op) => op.serialize(serializer),
            Self::UnsignedPLC(op) => op.serialize(serializer),
            Self::SignedPLC(op) => op.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PLCOperation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let json = match serde_json::to_string(&value) {
            Ok(json) => json,
            Err(e) => return Err(serde::de::Error::custom(e)),
        };
        let op = match op_from_json(json.as_str()) {
            Ok(op) => op,
            Err(e) => return Err(serde::de::Error::custom(e)),
        };
        Ok(op)
    }
}

impl Into<UnsignedGenesisOperation> for PLCOperation {
    fn into(self) -> UnsignedGenesisOperation {
        match self {
            Self::UnsignedGenesis(op) => op,
            _ => panic!("Not a UnsignedGenesisOperation"),
        }
    }
}

impl Into<SignedGenesisOperation> for PLCOperation {
    fn into(self) -> SignedGenesisOperation {
        match self {
            Self::SignedGenesis(op) => op,
            _ => panic!("Not a SignedGenesisOperation"),
        }
    }
}

impl Into<UnsignedPLCOperation> for PLCOperation {
    fn into(self) -> UnsignedPLCOperation {
        match self {
            Self::UnsignedPLC(op) => op,
            _ => panic!("Not a UnsignedPLCOperation"),
        }
    }
}

impl Into<SignedPLCOperation> for PLCOperation {
    fn into(self) -> SignedPLCOperation {
        match self {
            Self::SignedPLC(op) => op,
            _ => panic!("Not a SignedPLCOperation"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[serde(rename = "type")]
    pub type_: String,
    pub endpoint: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedPLCOperation {
    #[serde(rename = "type")]
    pub type_: PLCOperationType,
    pub rotation_keys: Vec<String>,
    pub verification_methods: HashMap<String, String>,
    pub also_known_as: Vec<String>,
    pub services: HashMap<String, Service>,
    pub prev: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignedPLCOperation {
    #[serde(flatten)]
    pub unsigned: UnsignedPLCOperation,
    pub sig: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedGenesisOperation {
    #[serde(rename = "type")]
    type_: String,
    pub signing_key: String,
    pub recovery_key: String,
    pub handle: String,
    pub service: String,
    pub prev: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignedGenesisOperation {
    #[serde(flatten)]
    pub unsigned: UnsignedGenesisOperation,
    pub sig: String,
}

impl UnsignedOperation for UnsignedPLCOperation {
    fn to_json(&self) -> String {
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

    fn to_signed(&self, key: &str) -> Result<impl SignedOperation, crate::Error> {
        let keypair = Keypair::from_private_key(key.to_string())?;
        let dag = serde_ipld_dagcbor::to_vec(&self).unwrap();

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let sig = engine.encode(keypair.sign(&dag.as_slice())?);

        Ok(SignedPLCOperation {
            unsigned: self.clone(),
            sig,
        })
    }
}

impl UnsignedGenesisOperation {
    pub fn normalize(&self) -> Result<PLCOperation, crate::Error> {
        let op = serde_json::to_value(self)?;
        let normalized = normalize_op(op);
        Ok(PLCOperation::UnsignedPLC(serde_json::from_value::<UnsignedPLCOperation>(normalized)?))
    }
}

impl UnsignedOperation for UnsignedGenesisOperation {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn to_signed(&self, key: &str) -> Result<impl SignedOperation, crate::Error> {
        let keypair = Keypair::from_private_key(key.to_string())?;
        let dag = serde_ipld_dagcbor::to_vec(&self).unwrap();

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let sig = engine.encode(keypair.sign(&dag.as_slice())?);

        Ok(SignedGenesisOperation {
            unsigned: self.clone(),
            sig,
        })
    }
}

impl SignedPLCOperation {
    pub fn from_json(json: &str) -> Result<Self, crate::Error> {
        let raw: serde_json::Value = serde_json::from_str(json)?;
        let mut raw = raw.as_object().unwrap().to_owned();
        let sig = match raw.get("sig") {
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => return Err(crate::Error::UnsignedOperation),
        };
        raw.remove("sig");
        let raw = normalize_op(serde_json::to_value(raw.clone())?);

        let unsigned: UnsignedPLCOperation = serde_json::from_value(raw.clone())?;
        Ok(Self { unsigned, sig })
    }
}

impl SignedOperation for SignedPLCOperation {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn to_cid(&self) -> Result<String, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let result = Code::Sha2_256.digest(&dag.as_slice());
        let cid = Cid::new_v1(0x71, result);
        Ok(cid.to_string())
    }

    fn to_did(&self) -> Result<String, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let hashed = Sha256::digest(dag.as_slice());
        let b32 = base32::encode(Alphabet::Rfc4648Lower { padding: false }, hashed.as_slice());
        Ok(format!("did:plc:{}", b32[0..24].to_string()))
    }

    fn verify_sig(&self) -> Result<bool, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self.unsigned) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let dag = dag.as_slice();

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let decoded_sig = engine.decode(self.sig.as_bytes())?;

        for key in &self.unsigned.rotation_keys {
            let keypair = Keypair::from_did_key(key.to_string())?;

            if keypair.verify(dag, &decoded_sig)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl SignedGenesisOperation {
    pub fn from_json(json: &str) -> Result<Self, crate::Error> {
        let raw: serde_json::Value = serde_json::from_str(json)?;
        let mut raw = raw.as_object().unwrap().to_owned();
        let sig = match raw.get("sig") {
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => return Err(crate::Error::UnsignedOperation),
        };
        raw.remove("sig");

        let unsigned: UnsignedGenesisOperation = serde_json::from_value(serde_json::to_value(raw.clone())?)?;
        Ok(Self { unsigned, sig })
    }

    pub fn normalize(&self) -> Result<PLCOperation, crate::Error> {
        let op = serde_json::to_value(self)?;
        let normalized = normalize_op(op);
        Ok(PLCOperation::SignedPLC(serde_json::from_value::<SignedPLCOperation>(normalized)?))
    }
}

impl SignedOperation for SignedGenesisOperation {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn to_cid(&self) -> Result<String, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let result = Code::Sha2_256.digest(&dag.as_slice());
        let cid = Cid::new_v1(0x71, result);
        Ok(cid.to_string())
    }

    fn to_did(&self) -> Result<String, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let hashed = Sha256::digest(dag.as_slice());
        let b32 = base32::encode(Alphabet::Rfc4648Lower { padding: false }, hashed.as_slice());
        Ok(format!("did:plc:{}", b32[0..24].to_string()))
    }

    fn verify_sig(&self) -> Result<bool, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self.unsigned) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let dag = dag.as_slice();

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let decoded_sig = engine.decode(self.sig.as_bytes())?;

        let rotation_keys = [&self.unsigned.recovery_key, &self.unsigned.signing_key];
        for key in rotation_keys {
            let keypair = Keypair::from_did_key(key.to_string())?;

            if keypair.verify(dag, &decoded_sig)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_OP_JSON: &str = "{\"sig\":\"8Wj9Cf74dZFNKx7oucZSHbBDFOMJ3xx9lkvj5rT9xMErssWYl1D9n4PeGC0mNml7xDG7uoQqZ1JWoApGADUgXg\",\"prev\":\"bafyreiexwziulimyiw3qlhpwr2zljk5jtzdp2bgqbgoxuemjsf5a6tan3a\",\"type\":\"plc_operation\",\"services\":{\"atproto_pds\":{\"type\":\"AtprotoPersonalDataServer\",\"endpoint\":\"https://puffball.us-east.host.bsky.network\"}},\"alsoKnownAs\":[\"at://bsky.app\"],\"rotationKeys\":[\"did:key:zQ3shhCGUqDKjStzuDxPkTxN6ujddP4RkEKJJouJGRRkaLGbg\",\"did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK\"],\"verificationMethods\":{\"atproto\":\"did:key:zQ3shQo6TF2moaqMTrUZEM1jeuYRQXeHEx4evX9751y2qPqRA\"}}";
    const TEST_GENESIS_OP_JSON: &str = "{\"sig\":\"9NuYV7AqwHVTc0YuWzNV3CJafsSZWH7qCxHRUIP2xWlB-YexXC1OaYAnUayiCXLVzRQ8WBXIqF-SvZdNalwcjA\",\"prev\":null,\"type\":\"plc_operation\",\"services\":{\"atproto_pds\":{\"type\":\"AtprotoPersonalDataServer\",\"endpoint\":\"https://bsky.social\"}},\"alsoKnownAs\":[\"at://bluesky-team.bsky.social\"],\"rotationKeys\":[\"did:key:zQ3shhCGUqDKjStzuDxPkTxN6ujddP4RkEKJJouJGRRkaLGbg\",\"did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK\"],\"verificationMethods\":{\"atproto\":\"did:key:zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF\"}}";
    const TEST_PREV_OP_JSON: &str = "{\"sig\":\"OoDJihYhLUEWp2MGiAoCN1sRj9cgUEqNjZe6FIOePB8Ugp-IWAZplFRm-pU-fbYSpYV1_tQ9Gx8d_PR9f3NBAg\",\"prev\":\"bafyreihmuvr3frdvd6vmdhucih277prdcfcezf67lasg5oekxoimnunjoq\",\"type\":\"plc_operation\",\"services\":{\"atproto_pds\":{\"type\":\"AtprotoPersonalDataServer\",\"endpoint\":\"https://bsky.social\"}},\"alsoKnownAs\":[\"at://bsky.app\"],\"rotationKeys\":[\"did:key:zQ3shhCGUqDKjStzuDxPkTxN6ujddP4RkEKJJouJGRRkaLGbg\",\"did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK\"],\"verificationMethods\":{\"atproto\":\"did:key:zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF\"}}";
    const TEST_DID: &str = "did:plc:z72i7hdynmk6r22z27h6tvur";

    #[actix_rt::test]
    async fn test_signed_to_json() {
        let op: SignedPLCOperation = SignedPLCOperation::from_json(TEST_OP_JSON).unwrap();
        let json = op.to_json();

        let object = serde_json::from_str::<serde_json::Value>(&json)
            .unwrap();
        let object = object.as_object()
            .unwrap();

        assert!(object.contains_key("sig"));
        assert!(object.contains_key("prev"));
        assert!(object.contains_key("type"));
        assert!(object.contains_key("services"));
        assert!(object.contains_key("alsoKnownAs"));
        assert!(object.contains_key("rotationKeys"));
        assert!(object.contains_key("verificationMethods"));
        assert!(object.get("type").unwrap() == "plc_operation");

        // Validate structure of rotationKeys
        let rotation_keys = object.get("rotationKeys").unwrap().as_array().unwrap();
        assert!(rotation_keys.len() == 2);
        for key in rotation_keys {
            assert!(key.is_string());
            match Keypair::from_did_key(key.as_str().unwrap().to_string()) {
                Ok(_) => {}
                Err(e) => panic!("{}", e),
            }
        }

        // Validate structure of verificationMethods
        let verification_methods = object.get("verificationMethods").unwrap().as_object().unwrap();
        assert!(verification_methods.len() == 1);
        assert!(verification_methods.contains_key("atproto"));
        for (key, value) in verification_methods {
            assert!(value.is_string());
            if key == "atproto" {
                match Keypair::from_did_key(value.as_str().unwrap().to_string()) {
                    Ok(_) => {}
                    Err(e) => panic!("{}", e),
                }
            }
        }

        // Validate structure of alsoKnownAs
        let also_known_as = object.get("alsoKnownAs").unwrap().as_array().unwrap();
        assert!(also_known_as.len() == 1);
        assert!(also_known_as[0].is_string());
        assert!(also_known_as[0].as_str().unwrap().starts_with("at://"));
        assert!(also_known_as[0].as_str().unwrap() == "at://bsky.app");

        // Validate structure of services
        let services = object.get("services").unwrap().as_object().unwrap();
        assert!(services.len() == 1);
        assert!(services.contains_key("atproto_pds"));
        let service = services.get("atproto_pds").unwrap().as_object().unwrap();
        assert!(service.len() == 2);
        assert!(service.contains_key("type"));
        assert!(service.contains_key("endpoint"));
        assert!(service.get("type").unwrap() == "AtprotoPersonalDataServer");
        assert!(service.get("endpoint").unwrap().as_str().unwrap().starts_with("https://"));
    }

    #[actix_rt::test]
    async fn test_to_cid() {
        let op: SignedPLCOperation = SignedPLCOperation::from_json(TEST_PREV_OP_JSON).unwrap();
        let cid = op.to_cid().unwrap();

        assert!(cid == "bafyreiexwziulimyiw3qlhpwr2zljk5jtzdp2bgqbgoxuemjsf5a6tan3a".to_string());
    }

    #[actix_rt::test]
    async fn test_to_did() {
        let op: SignedPLCOperation = SignedPLCOperation::from_json(TEST_GENESIS_OP_JSON).unwrap();
        let did = op.to_did().unwrap();

        assert!(did == TEST_DID.to_string());
    }

    #[actix_rt::test]
    async fn test_verify_sig() {
        let op: SignedPLCOperation = SignedPLCOperation::from_json(TEST_OP_JSON).unwrap();

        assert!(op.verify_sig().unwrap());
    }
}