#![allow(dead_code, unused_imports)]
use crate::database::RepoBlock;
use rsky_pds::repo::block_map::{BlockMap, BlocksAndMissing};
use rsky_pds::storage::{ObjAndBytes, CidAndRev};
use rsky_pds::repo::types::{CommitData, RepoRecord};
use anyhow::Result;
use lexicon_cid::Cid;
use libipld::cbor::encode::write_null;
use libipld::cbor::DagCborCodec;
use libipld::codec::Encode;
use serde_cbor::Value as CborValue;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::io::Write;
use serde::{Deserialize, Serialize};

mod surreal;

pub use surreal::SurrealRepoReader;

/// Ipld
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Ipld {
    /// Represents a Cid.
    Link(Cid),
    /// Represents a sequence of bytes.
    Bytes(Vec<u8>),
    /// Represents a list.
    List(Vec<Ipld>),
    /// Represents a map of strings to objects.
    Map(BTreeMap<String, Ipld>),
    /// String
    String(String),
    /// Represents a Json Value
    Json(JsonValue),
}

impl Encode<DagCborCodec> for Ipld {
    fn encode<W: Write>(&self, c: DagCborCodec, w: &mut W) -> Result<()> {
        match self {
            Self::Json(JsonValue::Null) => write_null(w),
            Self::Json(JsonValue::Bool(b)) => b.encode(c, w),
            Self::Json(JsonValue::Number(n)) => {
                if n.is_f64() {
                    n.as_f64().unwrap().encode(c, w)
                } else if n.is_u64() {
                    n.as_u64().unwrap().encode(c, w)
                } else {
                    n.as_i64().unwrap().encode(c, w)
                }
            }
            Self::Json(JsonValue::String(s)) => s.encode(c, w),
            Self::Json(JsonValue::Object(o)) => serde_json::to_vec(o)?.encode(c, w),
            Self::Json(JsonValue::Array(a)) => serde_json::to_vec(a)?.as_slice().encode(c, w),
            Self::Bytes(b) => b.as_slice().encode(c, w),
            Self::List(l) => l.encode(c, w),
            Self::Map(m) => m.encode(c, w),
            Self::Link(cid) => cid.encode(c, w),
            Self::String(s) => s.encode(c, w),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RepoReader {
    SurrealDB(SurrealRepoReader)
}

impl RepoReader {
    pub fn blocks(&mut self) -> &mut BlockMap {
        match self {
            RepoReader::SurrealDB(db) => &mut db.blocks,
        }
    }

    pub async fn get_blocks(&mut self, cids: Vec<Cid>) -> Result<BlocksAndMissing> {
        match self {
            RepoReader::SurrealDB(db) => db.get_blocks(cids).await,
        }
    }

    pub async fn get_car_stream(&self, since: Option<String>) -> Result<Vec<u8>> {
        match self {
            RepoReader::SurrealDB(db) => db.get_car_stream(since).await,
        }
    }

    pub async fn get_block_range(
        &self,
        since: &Option<String>,
        cursor: &Option<CidAndRev>,
    ) -> Result<Vec<RepoBlock>> {
        match self {
            RepoReader::SurrealDB(db) => db.get_block_range(since, cursor).await,
        }
    }

    pub async fn get_bytes(&mut self, cid: &Cid) -> Result<Vec<u8>> {
        match self {
            RepoReader::SurrealDB(db) => db.get_bytes(cid).await,
        }
    }

    pub async fn count_blocks(&self) -> Result<i64> {
        match self {
            RepoReader::SurrealDB(db) => db.count_blocks().await,
        }
    }

    pub async fn has(&mut self, cid: Cid) -> Result<bool> {
        match self {
            RepoReader::SurrealDB(db) => db.has(cid).await,
        }
    }

    pub async fn attempt_read(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        match self {
            RepoReader::SurrealDB(db) => db.attempt_read(cid, check).await,
        }
    }

    pub async fn read_obj_and_bytes(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        match self {
            RepoReader::SurrealDB(db) => db.read_obj_and_bytes(cid, check).await,
        }
    }

    pub async fn read_obj(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<CborValue> {
        match self {
            RepoReader::SurrealDB(db) => db.read_obj(cid, check).await,
        }
    }

    pub async fn read_record(&mut self, cid: &Cid) -> Result<RepoRecord> {
        match self {
            RepoReader::SurrealDB(db) => db.read_record(cid).await,
        }
    }

    pub async fn cache_rev(&mut self, rev: String) -> Result<()> {
        match self {
            RepoReader::SurrealDB(db) => db.cache_rev(rev).await,
        }
    }

    pub async fn apply_commit(
        &mut self,
        commit: CommitData,
        is_create: Option<bool>,
    ) -> Result<()> {
        match self {
            RepoReader::SurrealDB(db) => db.apply_commit(commit, is_create).await,
        }
    }

    pub async fn put_many(&self, to_put: BlockMap, rev: String) -> Result<()> {
        match self {
            RepoReader::SurrealDB(db) => db.put_many(to_put, rev).await,
        }
    }

    pub async fn delete_many(&self, cids: Vec<Cid>) -> Result<()> {
        match self {
            RepoReader::SurrealDB(db) => db.delete_many(cids).await,
        }
    }

    pub async fn update_root(&self, cid: Cid, rev: String, is_create: Option<bool>) -> Result<()> {
        match self {
            RepoReader::SurrealDB(db) => db.update_root(cid, rev, is_create).await,
        }
    }

    pub async fn get_root(&self) -> Option<Cid> {
        match self {
            RepoReader::SurrealDB(db) => db.get_root().await,
        }
    }

    pub async fn get_root_detailed(&self) -> Result<CidAndRev> {
        match self {
            RepoReader::SurrealDB(db) => db.get_root_detailed().await,
        }
    }
}