use multihash_codetable::{Code, MultihashDigest};
use ecdsa::signature::{SignerMut, Verifier};
use serde::{Deserialize, Serialize};
use crate::multicodec::MultiEncoded;
use std::collections::HashMap;
use sha2::{Digest, Sha256};
use base32::Alphabet;
use base64::Engine;
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

    pub fn to_signed(&self, key: &str) -> Result<SignedPLCOperation, crate::Error> {
        let (_base, data) = multibase::decode(key)?;
        let dag = serde_ipld_dagcbor::to_vec(&self).unwrap();

        if key.starts_with("zDn") {
            // P-256
            let mut sk = p256::ecdsa::SigningKey::from_slice(data.as_slice()).unwrap();
            let sig: p256::ecdsa::Signature = sk.sign(&dag.as_slice());
            let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
            let sig = engine.encode(sig.to_bytes());
            return Ok(SignedPLCOperation {
                unsigned: self.clone(),
                sig,
            });
        } else if key.starts_with("zQ3s") {
            // Secp256k1
            let mut sk = k256::ecdsa::SigningKey::from_slice(data.as_slice()).unwrap();
            let sig: k256::ecdsa::Signature = sk.sign(&dag.as_slice());
            let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
            let sig = engine.encode(sig.to_bytes());
            return Ok(SignedPLCOperation {
                unsigned: self.clone(),
                sig,
            });
        }
        Err(crate::Error::InvalidKey)
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

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_cid(&self) -> Result<String, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let result = Code::Sha2_256.digest(&dag.as_slice());
        let cid = Cid::new_v1(0x71, result);
        Ok(cid.to_string())
    }

    pub fn to_did(&self) -> Result<String, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let hashed = Sha256::digest(dag.as_slice());
        let b32 = base32::encode(Alphabet::Rfc4648Lower { padding: false }, hashed.as_slice());
        Ok(format!("did:plc:{}", b32[0..24].to_string()))
    }

    pub fn verify_sig(&self) -> Result<bool, crate::Error> {
        let dag = match serde_ipld_dagcbor::to_vec(&self.unsigned) {
            Ok(dag) => dag,
            Err(e) => return Err(crate::Error::DagCbor(e.to_string())),
        };
        let dag = dag.as_slice();

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let decoded_sig = engine.decode(self.sig.as_bytes())?;

        for key in &self.unsigned.rotation_keys {
            let key = key.split_at(8).1;
            let (_base, data) = multibase::decode(key)?;
            let decoded_result = MultiEncoded::new(data.as_slice())?;

            match decoded_result.codec() {
                0xe7 => {
                    // Secp256k1
                    let vk = k256::ecdsa::VerifyingKey::from_sec1_bytes(decoded_result.data())?;
                    let sig = k256::ecdsa::Signature::from_slice(decoded_sig.as_slice().into())?;
                    if vk.verify(dag, &sig).is_ok() {
                        return Ok(true);
                    }
                },
                0x1200 => {
                    // P-256
                    let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(decoded_result.data())?;
                    let sig = p256::ecdsa::Signature::from_slice(decoded_sig.as_slice().into())?;
                    if vk.verify(dag, &sig).is_ok() {
                        return Ok(true);
                    }
                },
                _ => continue,
            }

            // if key.starts_with("zDn") {
            //     // P-256
            //     let point =
            //         p256::EncodedPoint::from_bytes(&data[4..35])?;
            //     let pk = p256::ecdsa::VerifyingKey::from_encoded_point(&point)?;
            //     let sig = p256::ecdsa::Signature::from_bytes(self.sig.as_bytes().into())?;
            //     if pk.verify(&dag, &sig).is_ok() {
            //         return Ok(true);
            //     }
            // } else if key.starts_with("zQ3s") {
            //     // Secp256k1
            //     let point =
            //         k256::EncodedPoint::from_bytes(&data[2..35])?;
            //     let vk = k256::ecdsa::VerifyingKey::from_encoded_point(&point)?;
            //     let sig = k256::ecdsa::Signature::from_bytes(self.sig.as_bytes().into())?;
            //     if vk.verify(&dag, &sig).is_ok() {
            //         return Ok(true);
            //     }
            // } else {
            //     continue;
            // }
        }
        Ok(false)
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

        println!("{}", json);
        println!("{}", TEST_OP_JSON);

        assert!(json == TEST_OP_JSON.to_string());
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
