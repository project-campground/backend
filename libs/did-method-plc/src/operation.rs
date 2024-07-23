use crate::util::normalize_op;
use crate::util::op_from_json;
use crate::Keypair;
use crate::PLCError;
use base32::Alphabet;
use base64::Engine;
use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Clone)]
pub enum PLCOperationType {
    Operation,
    Tombstone,
}

impl PLCOperationType {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Operation => "plc_operation",
            Self::Tombstone => "plc_tombstone",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "plc_operation" => Some(Self::Operation),
            "plc_tombstone" => Some(Self::Tombstone),
            "create" => Some(Self::Operation),
            _ => None,
        }
    }
}

impl PartialEq for PLCOperationType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
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
    fn to_signed(&self, key: &str) -> Result<impl SignedOperation, PLCError>;
}

pub trait SignedOperation {
    fn to_json(&self) -> String;
    fn to_cid(&self) -> Result<String, PLCError>;
    fn to_did(&self) -> Result<String, PLCError>;
    fn verify_sig(
        &self,
        rotation_keys: Option<Vec<String>>,
    ) -> Result<(bool, Option<String>), PLCError>;
}

#[derive(Clone)]
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
        D: serde::Deserializer<'de>,
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
    pub type_: String,
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

    #[allow(refining_impl_trait)]
    fn to_signed(&self, key: &str) -> Result<SignedPLCOperation, PLCError> {
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
    pub fn normalize(&self) -> Result<PLCOperation, PLCError> {
        let op = serde_json::to_value(self).map_err(|e| PLCError::Other(e.into()))?;
        let normalized = normalize_op(op);
        Ok(PLCOperation::UnsignedPLC(
            serde_json::from_value::<UnsignedPLCOperation>(normalized)
                .map_err(|e| PLCError::Other(e.into()))?,
        ))
    }
}

impl UnsignedOperation for UnsignedGenesisOperation {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    #[allow(refining_impl_trait)]
    fn to_signed(&self, key: &str) -> Result<SignedGenesisOperation, PLCError> {
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
    pub fn from_json(json: &str) -> Result<Self, PLCError> {
        let raw: serde_json::Value = serde_json::from_str(json).map_err(|e| PLCError::Other(e.into()))?;
        let mut raw = raw.as_object().unwrap().to_owned();
        let sig = match raw.get("sig") {
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => return Err(PLCError::InvalidSignature),
        };
        raw.remove("sig");
        let raw = normalize_op(serde_json::to_value(raw.clone()).map_err(|e| PLCError::Other(e.into()))?);

        let unsigned: UnsignedPLCOperation = serde_json::from_value(raw.clone()).map_err(|e| PLCError::Other(e.into()))?;
        Ok(Self { unsigned, sig })
    }
}

impl SignedOperation for SignedPLCOperation {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn to_cid(&self) -> Result<String, PLCError> {
        let dag = serde_ipld_dagcbor::to_vec(&self).map_err(|e| PLCError::Other(e.into()))?;
        let result = Code::Sha2_256.digest(&dag.as_slice());
        let cid = Cid::new_v1(0x71, result);
        Ok(cid.to_string())
    }

    fn to_did(&self) -> Result<String, PLCError> {
        let dag = serde_ipld_dagcbor::to_vec(&self).map_err(|e| PLCError::Other(e.into()))?;
        let hashed = Sha256::digest(dag.as_slice());
        let b32 = base32::encode(Alphabet::Rfc4648Lower { padding: false }, hashed.as_slice());
        Ok(format!("did:plc:{}", b32[0..24].to_string()))
    }

    fn verify_sig(
        &self,
        rotation_keys: Option<Vec<String>>,
    ) -> Result<(bool, Option<String>), PLCError> {
        let dag =
            serde_ipld_dagcbor::to_vec(&self.unsigned).map_err(|e| PLCError::Other(e.into()))?;
        let dag = dag.as_slice();

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let decoded_sig = engine
            .decode(self.sig.as_bytes())
            .map_err(|_| PLCError::InvalidSignature)?;

        let rotation_keys = match rotation_keys {
            Some(keys) => keys.clone(),
            None => self.unsigned.rotation_keys.clone(),
        };

        for key in rotation_keys {
            let keypair =
                Keypair::from_did_key(key.to_string()).map_err(|_| PLCError::InvalidOperation)?;

            if keypair
                .verify(dag, &decoded_sig)
                .map_err(|e| PLCError::Other(e.into()))?
            {
                return Ok((true, Some(key.to_string())));
            }
        }
        Ok((false, None))
    }
}

impl SignedGenesisOperation {
    pub fn from_json(json: &str) -> Result<Self, PLCError> {
        let raw: serde_json::Value =
            serde_json::from_str(json).map_err(|e| PLCError::Other(e.into()))?;
        let mut raw = raw.as_object().unwrap().to_owned();
        let sig = match raw.get("sig") {
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => return Err(PLCError::InvalidSignature),
        };
        raw.remove("sig");

        let unsigned: UnsignedGenesisOperation = serde_json::from_value(
            serde_json::to_value(raw.clone()).map_err(|e| PLCError::Other(e.into()))?,
        )
        .map_err(|e| PLCError::Other(e.into()))?;
        Ok(Self { unsigned, sig })
    }

    pub fn normalize(&self) -> Result<PLCOperation, PLCError> {
        let op = serde_json::to_value(self).map_err(|e| PLCError::Other(e.into()))?;
        let normalized = normalize_op(op);
        Ok(PLCOperation::SignedPLC(
            serde_json::from_value::<SignedPLCOperation>(normalized)
                .map_err(|e| PLCError::Other(e.into()))?,
        ))
    }
}

impl SignedOperation for SignedGenesisOperation {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn to_cid(&self) -> Result<String, PLCError> {
        let dag = serde_ipld_dagcbor::to_vec(&self).map_err(|e| PLCError::Other(e.into()))?;
        let result = Code::Sha2_256.digest(&dag.as_slice());
        let cid = Cid::new_v1(0x71, result);
        Ok(cid.to_string())
    }

    fn to_did(&self) -> Result<String, PLCError> {
        let dag = serde_ipld_dagcbor::to_vec(&self).map_err(|e| PLCError::Other(e.into()))?;
        let hashed = Sha256::digest(dag.as_slice());
        let b32 = base32::encode(Alphabet::Rfc4648Lower { padding: false }, hashed.as_slice());
        Ok(format!("did:plc:{}", b32[0..24].to_string()))
    }

    fn verify_sig(
        &self,
        rotation_keys: Option<Vec<String>>,
    ) -> Result<(bool, Option<String>), PLCError> {
        let dag =
            serde_ipld_dagcbor::to_vec(&self.unsigned).map_err(|e| PLCError::Other(e.into()))?;
        let dag = dag.as_slice();

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let decoded_sig = engine
            .decode(self.sig.as_bytes())
            .map_err(|_| PLCError::InvalidSignature)?;

        let rotation_keys = match rotation_keys {
            Some(keys) => keys.clone(),
            None => [
                self.unsigned.recovery_key.clone(),
                self.unsigned.signing_key.clone(),
            ]
            .to_vec(),
        };
        for key in rotation_keys {
            let keypair =
                Keypair::from_did_key(key.to_string()).map_err(|_| PLCError::InvalidOperation)?;

            if keypair
                .verify(dag, &decoded_sig)
                .map_err(|e| PLCError::Other(e.into()))?
            {
                return Ok((true, Some(key.to_string())));
            }
        }
        Ok((false, None))
    }
}

#[cfg(test)]
mod tests {
    use crate::BlessedAlgorithm;

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
        let (valid, key) = op.verify_sig(None).unwrap();

        assert!(valid);
        assert!(key.unwrap() == "did:key:zQ3shpKnbdPx3g3CmPf5cRVTPe1HtSwVn5ish3wSnDPQCbLJK".to_string());
    }

    #[actix_rt::test]
    async fn test_to_signed() {
        let signing_key = Keypair::generate(BlessedAlgorithm::P256);
        let recovery_key = Keypair::generate(BlessedAlgorithm::P256);
        let validation_key = Keypair::generate(BlessedAlgorithm::P256);

        let op = UnsignedPLCOperation {
            prev: None,
            type_: PLCOperationType::Operation,
            services: HashMap::from([
                ("atproto_pds".to_string(), Service {
                    type_: "AtprotoPersonalDataServer".to_string(),
                    endpoint: "https://example.test".to_string(),
                }),
            ]),
            also_known_as: vec!["at://example.test".to_string()],
            rotation_keys: vec![
                recovery_key.to_did_key().unwrap(),
                signing_key.to_did_key().unwrap(),
            ],
            verification_methods: HashMap::from([
                ("atproto".to_string(), validation_key.to_did_key().unwrap()),
            ]),
        };
        let signed = op.to_signed(signing_key.to_private_key().unwrap().as_str()).unwrap();
        let (valid, key) = signed.verify_sig(None).unwrap();
        assert!(valid);
        assert!(key.unwrap() == signing_key.to_did_key().unwrap());
    }
}
