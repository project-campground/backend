#![allow(dead_code, unused_imports)]
use crate::database::RepoBlock;
use rsky_pds::repo::block_map::{BlockMap, BlocksAndMissing};
use rsky_pds::storage::{ObjAndBytes, CidAndRev};
use rsky_pds::repo::types::{CommitData, RepoRecord};
use anyhow::Result;
use lexicon_cid::Cid;
use serde_cbor::Value as CborValue;

// mod surreal;
mod postgres;

// pub use surreal::SurrealRepoReader;
pub use postgres::PostgresRepoReader;

#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub enum RepoReader {
    Postgres(PostgresRepoReader),
}

impl RepoReader {
    pub fn blocks(&mut self) -> &mut BlockMap {
        match self {
            RepoReader::Postgres(db) => &mut db.blocks,
        }
    }

    pub async fn get_blocks(&mut self, cids: Vec<Cid>) -> Result<BlocksAndMissing> {
        match self {
            RepoReader::Postgres(db) => db.get_blocks(cids).await,
        }
    }

    pub async fn get_car_stream(&self, since: Option<String>) -> Result<Vec<u8>> {
        match self {
            RepoReader::Postgres(db) => db.get_car_stream(since).await,   
        }
    }

    pub async fn get_block_range(
        &self,
        since: &Option<String>,
        cursor: &Option<CidAndRev>,
    ) -> Result<Vec<RepoBlock>> {
        match self {
            RepoReader::Postgres(db) => db.get_block_range(since, cursor).await,
        }
    }

    pub async fn get_bytes(&mut self, cid: &Cid) -> Result<Vec<u8>> {
        match self {
            RepoReader::Postgres(db) => db.get_bytes(cid).await,
        }
    }

    pub async fn count_blocks(&self) -> Result<i64> {
        match self {
            RepoReader::Postgres(db) => db.count_blocks().await,
        }
    }

    pub async fn has(&mut self, cid: Cid) -> Result<bool> {
        match self {
            RepoReader::Postgres(db) => db.has(cid).await,
        }
    }

    pub async fn attempt_read(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        match self {
            RepoReader::Postgres(db) => db.attempt_read(cid, check).await,
        }
    }

    pub async fn read_obj_and_bytes(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        match self {
            RepoReader::Postgres(db) => db.read_obj_and_bytes(cid, check).await,
        }
    }

    pub async fn read_obj(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<CborValue> {
        match self {
            RepoReader::Postgres(db) => db.read_obj(cid, check).await,
        }
    }

    pub async fn read_record(&mut self, cid: &Cid) -> Result<RepoRecord> {
        match self {
            RepoReader::Postgres(db) => db.read_record(cid).await,
        }
    }

    pub async fn cache_rev(&mut self, rev: String) -> Result<()> {
        match self {
            RepoReader::Postgres(db) => db.cache_rev(rev).await,
        }
    }

    pub async fn apply_commit(
        &mut self,
        commit: CommitData,
        is_create: Option<bool>,
    ) -> Result<()> {
        match self {
            RepoReader::Postgres(db) => db.apply_commit(commit, is_create).await,
        }
    }

    pub async fn put_many(&self, to_put: BlockMap, rev: String) -> Result<()> {
        match self {
            RepoReader::Postgres(db) => db.put_many(to_put, rev).await,
        }
    }

    pub async fn delete_many(&self, cids: Vec<Cid>) -> Result<()> {
        match self {
            RepoReader::Postgres(db) => db.delete_many(cids).await,
        }
    }

    pub async fn update_root(&self, cid: Cid, rev: String, is_create: Option<bool>) -> Result<()> {
        match self {
            RepoReader::Postgres(db) => db.update_root(cid, rev, is_create).await,
        }
    }

    pub async fn get_root(&self) -> Option<Cid> {
        match self {
            RepoReader::Postgres(db) => db.get_root().await,
        }
    }

    pub async fn get_root_detailed(&self) -> Result<CidAndRev> {
        match self {
            RepoReader::Postgres(db) => db.get_root_detailed().await,
        }
    }
}