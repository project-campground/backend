use rsky_pds::common::sign::sign_without_indexmap;
use rsky_pds::common::tid::Ticker;
use crate::repository::types::{Commit, Lex, RecordPath, RepoRecord, UnsignedCommit, VersionedCommit};
use rsky_pds::storage::Ipld;
use anyhow::{bail, Result};
use lexicon_cid::Cid;
use secp256k1::Keypair;
use serde_json::{json, Value as JsonValue};
use serde_cbor::Value as CborValue;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

use super::types::IpldCid;

pub fn sign_commit(unsigned: UnsignedCommit, keypair: Keypair) -> Result<Commit> {
    let commit_sig = sign_without_indexmap(&unsigned, &keypair.secret_key())?;
    Ok(Commit {
        did: unsigned.did,
        version: unsigned.version,
        data: unsigned.data,
        rev: unsigned.rev,
        prev: unsigned.prev,
        sig: commit_sig.to_vec(),
    })
}

pub fn format_data_key<T: FromStr + Display>(collection: T, rkey: T) -> String {
    format!("{collection}/{rkey}")
}

pub fn lex_to_ipld(val: Lex) -> Ipld {
    match val {
        Lex::List(list) => Ipld::List(
            list.into_iter()
                .map(|item| lex_to_ipld(item))
                .collect::<Vec<Ipld>>(),
        ),
        Lex::Map(map) => {
            let mut to_return: BTreeMap<String, Ipld> = BTreeMap::new();
            for key in map.keys() {
                to_return.insert(key.to_owned(), lex_to_ipld(map.get(key).unwrap().clone()));
            }
            Ipld::Map(to_return)
        }
        Lex::Blob(blob) => {
            Ipld::Json(serde_json::to_value(blob.original).expect("Issue serializing blob"))
        }
        Lex::Ipld(ipld) => match ipld {
            Ipld::Json(json_val) => match serde_json::from_value::<Cid>(json_val.clone()) {
                Ok(cid) => Ipld::Link(cid),
                Err(_) => Ipld::Json(json_val),
            },
            _ => ipld,
        },
    }
}

pub fn ipld_to_lex(val: Ipld) -> Lex {
    match val {
        Ipld::List(list) => Lex::List(
            list.into_iter()
                .map(|item| ipld_to_lex(item))
                .collect::<Vec<Lex>>(),
        ),
        Ipld::Map(map) => {
            let mut to_return: BTreeMap<String, Lex> = BTreeMap::new();
            for key in map.keys() {
                to_return.insert(key.to_owned(), ipld_to_lex(map.get(key).unwrap().clone()));
            }
            Lex::Map(to_return)
        }
        Ipld::Json(blob)
            if blob.get("$type") == Some(&JsonValue::String("blob".to_string()))
                || (matches!(blob.get("cid"), Some(&JsonValue::String(_)))
                    && matches!(blob.get("mimeType"), Some(&JsonValue::String(_)))) =>
        {
            Lex::Blob(serde_json::from_value(blob).expect("Issue deserializing blob"))
        }
        _ => Lex::Ipld(val),
    }
}

pub fn cbor_to_lex(val: Vec<u8>) -> Result<Lex> {
    let obj: Ipld = serde_ipld_dagcbor::from_slice(val.as_slice())?; //cbordecode
    Ok(ipld_to_lex(obj))
}

pub fn cbor_to_lex_record(val: Vec<u8>) -> Result<RepoRecord> {
    let parsed = cbor_to_lex(val)?;
    match parsed {
        Lex::Map(map) => Ok(map),
        _ => bail!("Lexicon record should be a json object"),
    }
}

pub fn parse_data_key(key: &String) -> Result<RecordPath> {
    let parts: Vec<&str> = key.split("/").collect();
    if parts.len() != 2 {
        bail!("Invalid record key: `{key:?}`");
    }
    Ok(RecordPath {
        collection: parts[0].to_owned(),
        rkey: parts[1].to_owned(),
    })
}

pub fn ensure_v3_commit(commit: VersionedCommit) -> Commit {
    match commit {
        VersionedCommit::Commit(commit) if commit.version == 3 => commit,
        VersionedCommit::Commit(commit) => Commit {
            did: commit.did,
            version: 3,
            data: commit.data,
            rev: commit.rev,
            prev: commit.prev,
            sig: commit.sig,
        },
        VersionedCommit::LegacyV2Commit(commit) => Commit {
            did: commit.did,
            version: 3,
            data: IpldCid::from(commit.data),
            rev: commit.rev.unwrap_or(Ticker::new().next(None).to_string()),
            prev: match commit.prev {
                Some(prev) => Some(IpldCid::from(prev)),
                None => None,
            },
            sig: commit.sig,
        },
    }
}

pub fn cbor_value_to_ipld(val: CborValue) -> Ipld {
    match val {
        CborValue::Bool(b) => Ipld::Json(JsonValue::Bool(b)),
        CborValue::Integer(i) => Ipld::Json(json!(i)),
        CborValue::Float(f) => Ipld::Json(json!(f)),
        CborValue::Bytes(b) => Ipld::Bytes(b),
        CborValue::Text(t) => Ipld::String(t),
        CborValue::Array(a) => Ipld::List(
            a.into_iter()
                .map(|item| cbor_value_to_ipld(item))
                .collect::<Vec<Ipld>>(),
        ),
        CborValue::Map(m) => {
            let mut to_return: BTreeMap<String, Ipld> = BTreeMap::new();
            for key in m.keys() {
                let key_str = match key {
                    CborValue::Text(t) => t,
                    _ => continue,
                };
                to_return.insert(
                    key_str.clone(),
                    cbor_value_to_ipld(m.get(key).unwrap().clone()),
                );
            }
            Ipld::Map(to_return)
        }
        CborValue::Null => Ipld::Json(JsonValue::Null),
        _ => Ipld::Json(JsonValue::Null),
    }
}

pub fn deserialize_ipld_cbor(bytes: Vec<u8>) -> Result<BTreeMap<CborValue, Ipld>> {
    let obj: CborValue = serde_ipld_dagcbor::from_slice(bytes.as_slice())?;
    if let CborValue::Map(map) = obj {
        let mut altered: BTreeMap<CborValue, Ipld> = BTreeMap::new();
        // Change any Bytes values to CIDs if valid
        for key in map.keys() {
            let val = map.get(key).unwrap();
            match val {
                CborValue::Bytes(bytes) => {
                    match Cid::try_from(bytes.as_slice()) {
                        Ok(cid) => altered.insert(key.clone(), Ipld::Link(cid)),
                        Err(_) => altered.insert(key.clone(), Ipld::Bytes(bytes.clone())),
                    }
                }
                _ => altered.insert(key.clone(), cbor_value_to_ipld(val.clone())),
            };
        }
        Ok(altered)
    } else {
        bail!("Not a CBOR map");
    }
}
