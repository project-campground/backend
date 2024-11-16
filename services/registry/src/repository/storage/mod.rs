#![allow(dead_code, unused_imports)]
use rsky_pds::repo::block_map::{BlockMap, BlocksAndMissing};
use rsky_pds::storage::{ObjAndBytes, CidAndRev};
use rsky_pds::repo::types::{CommitData, RepoRecord};
use serde_cbor::Value as CborValue;
use std::str::FromStr;
use anyhow::{bail, Result};
use futures::try_join;
use lexicon_cid::Cid;
use crate::database::{establish_connection, RepoBlock, RepoRoot, models};
use rsky_pds::car::read_car_bytes;
use rsky_pds::common;
use rsky_pds::repo::error::DataStoreError;
use rsky_pds::storage::RepoRootError::RepoRootNotFoundError;
use rsky_pds::repo::cid_set::CidSet;
use rsky_pds::repo::util::cbor_to_lex_record;
use rsky_pds::repo::parse;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Text};
use diesel::*;
use serde::{Serialize, Deserialize};

#[allow(missing_debug_implementations)]
#[derive(Clone, Debug)]
pub struct RepoReader {
    pub cache: BlockMap,
    pub blocks: BlockMap,
    pub root: Option<Cid>,
    pub rev: Option<String>,
    pub now: String,
    pub did: String,
}

impl RepoReader {
    pub fn new(blocks: Option<BlockMap>, did: String, now: Option<String>) -> Self {
        let now = now.unwrap_or_else(|| common::now());
        let mut this = RepoReader {
            cache: BlockMap::new(),
            blocks: BlockMap::new(),
            root: None,
            rev: None,
            now,
            did,
        };
        if let Some(blocks) = blocks {
            this.blocks.add_map(blocks).unwrap();
        }
        this
    }

    pub async fn get_blocks(&mut self, cids: Vec<Cid>) -> Result<BlocksAndMissing> {
        use crate::schema::registry::repo_block::dsl as RepoBlockSchema;
        let conn = &mut establish_connection()?;

        let cached = self.cache.get_many(cids)?;

        if cached.missing.len() < 1 {
            return Ok(cached);
        }
        let mut missing = CidSet::new(Some(cached.missing.clone()));
        let missing_strings: Vec<String> =
            cached.missing.into_iter().map(|c| c.to_string()).collect();

        let mut blocks = BlockMap::new();

        let _: Vec<_> = missing_strings
            .chunks(500)
            .into_iter()
            .map(|batch| {
                let _: Vec<_> = RepoBlockSchema::repo_block
                    .filter(RepoBlockSchema::cid.eq_any(batch))
                    .filter(RepoBlockSchema::did.eq(&self.did))
                    .select((RepoBlockSchema::cid, RepoBlockSchema::content))
                    .load::<(String, Vec<u8>)>(conn)?
                    .into_iter()
                    .map(|row: (String, Vec<u8>)| {
                        let cid = Cid::from_str(&row.0).unwrap();
                        blocks.set(cid, row.1);
                        missing.delete(cid);
                    })
                    .collect();
                Ok::<(), anyhow::Error>(())
            })
            .collect();
        self.cache.add_map(blocks.clone())?;
        blocks.add_map(cached.blocks)?;
        Ok(BlocksAndMissing {
            blocks,
            missing: missing.to_list(),
        })
    }

    pub async fn get_car_stream(&self, since: Option<String>) -> Result<Vec<u8>> {
        match self.get_root().await {
            None => return Err(anyhow::Error::new(RepoRootNotFoundError)),
            Some(root) => {
                let mut car = BlockMap::new();
                let mut cursor: Option<CidAndRev> = None;
                let mut write_rows = |rows: Vec<RepoBlock>| -> Result<()> {
                    for row in rows {
                        car.set(Cid::from_str(&row.cid)?, row.content);
                    }
                    Ok(())
                };
                loop {
                    let res = self.get_block_range(&since, &cursor)?;
                    write_rows(res.clone())?;
                    if let Some(last_row) = res.last() {
                        cursor = Some(CidAndRev {
                            cid: Cid::from_str(&last_row.cid)?,
                            rev: last_row.repo_rev.clone(),
                        });
                    } else {
                        break;
                    }
                }
                read_car_bytes(Some(&root), car).await
            }
        }
    }

    pub fn get_block_range(
        &self,
        since: &Option<String>,
        cursor: &Option<CidAndRev>,
    ) -> Result<Vec<RepoBlock>> {
        use crate::schema::registry::repo_block::dsl as RepoBlockSchema;
        let conn = &mut establish_connection()?;

        let mut builder = RepoBlockSchema::repo_block
            .select(RepoBlock::as_select())
            .order((RepoBlockSchema::repoRev.desc(), RepoBlockSchema::cid.desc()))
            .filter(RepoBlockSchema::did.eq(&self.did))
            .limit(500)
            .into_boxed();

        if let Some(cursor) = cursor {
            // use this syntax to ensure we hit the index
            builder = builder.filter(
                sql::<Bool>("((")
                    .bind(RepoBlockSchema::repoRev)
                    .sql(", ")
                    .bind(RepoBlockSchema::cid)
                    .sql(") < (")
                    .bind::<Text, _>(cursor.rev.clone())
                    .sql(", ")
                    .bind::<Text, _>(cursor.cid.to_string())
                    .sql("))"),
            );
        }
        if let Some(since) = since {
            builder = builder.filter(RepoBlockSchema::repoRev.gt(since));
        }
        Ok(builder.load(conn)?)
    }

    pub fn get_bytes(&mut self, cid: &Cid) -> Result<Vec<u8>> {
        use crate::schema::registry::repo_block::dsl as RepoBlockSchema;
        let conn = &mut establish_connection()?;

        let cached = self.cache.get(*cid);
        if let Some(cached_result) = cached {
            return Ok(cached_result.clone());
        }

        let result: Vec<u8> = RepoBlockSchema::repo_block
            .filter(RepoBlockSchema::cid.eq(cid.to_string()))
            .filter(RepoBlockSchema::did.eq(&self.did))
            .select(RepoBlockSchema::content)
            .first(conn)
            .map_err(|_| anyhow::Error::new(DataStoreError::MissingBlock(cid.to_string())))?;
        self.cache.set(*cid, result.clone());
        Ok(result)
    }

    pub async fn count_blocks(&self) -> Result<i64> {
        use crate::schema::registry::repo_block::dsl as RepoBlockSchema;
        let conn = &mut establish_connection()?;

        let res = RepoBlockSchema::repo_block
            .filter(RepoBlockSchema::did.eq(&self.did))
            .count()
            .get_result(conn)?;
        Ok(res)
    }

    pub fn has(&mut self, cid: Cid) -> Result<bool> {
        let got = self.get_bytes(&cid);
        match got {
            Ok(got) => Ok(!got.is_empty()),
            Err(_) => Ok(false),
        }
    }

    pub fn attempt_read(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        let bytes = self.get_bytes(cid)?;
        Ok(parse::parse_obj_by_kind(bytes, *cid, check)?)
    }

    pub fn read_obj_and_bytes(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        let read = self.attempt_read(cid, check)?;
        Ok(read)
    }

    pub fn read_obj(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<CborValue> {
        let obj = self.read_obj_and_bytes(cid, check)?;
        Ok(obj.obj)
    }

    pub fn read_record(&mut self, cid: &Cid) -> Result<RepoRecord> {
        let bytes = self.get_bytes(cid)?;
        Ok(cbor_to_lex_record(bytes)?)
    }

    // Transactors
    // -------------------

    /// Proactively cache all blocks from a particular commit (to prevent multiple roundtrips)
    pub async fn cache_rev(&mut self, rev: String) -> Result<()> {
        use crate::schema::registry::repo_block::dsl as RepoBlockSchema;
        let conn = &mut establish_connection()?;

        let res: Vec<(String, Vec<u8>)> = RepoBlockSchema::repo_block
            .filter(RepoBlockSchema::did.eq(&self.did))
            .filter(RepoBlockSchema::repoRev.eq(rev))
            .select((RepoBlockSchema::cid, RepoBlockSchema::content))
            .limit(15)
            .get_results::<(String, Vec<u8>)>(conn)?;
        for row in res {
            self.cache.set(Cid::from_str(&row.0)?, row.1)
        }
        Ok(())
    }

    pub async fn apply_commit(
        &mut self,
        commit: CommitData,
        is_create: Option<bool>,
    ) -> Result<()> {
        try_join!(
            self.update_root(commit.cid, commit.rev.clone(), is_create),
            self.put_many(commit.new_blocks, commit.rev),
            self.delete_many(commit.removed_cids.to_list())
        )?;
        Ok(())
    }

    pub async fn put_many(&self, to_put: BlockMap, rev: String) -> Result<()> {
        use crate::schema::registry::repo_block::dsl as RepoBlockSchema;
        let conn = &mut establish_connection()?;

        let mut blocks: Vec<RepoBlock> = Vec::new();
        for (cid, bytes) in to_put.map.iter() {
            blocks.push(RepoBlock {
                cid: cid.to_string(),
                did: self.did.clone(),
                repo_rev: rev.clone(),
                size: bytes.len() as i32,
                content: bytes.clone(),
            });
        }
        let _ = blocks
            .chunks(50)
            .map(|batch| {
                Ok(insert_into(RepoBlockSchema::repo_block)
                    .values(batch)
                    .on_conflict_do_nothing()
                    .execute(conn)?)
            })
            .collect::<Result<Vec<usize>>>()?;
        Ok(())
    }

    pub async fn delete_many(&self, cids: Vec<Cid>) -> Result<()> {
        if cids.len() < 1 {
            return Ok(());
        }
        use crate::schema::registry::repo_block::dsl as RepoBlockSchema;
        let conn = &mut establish_connection()?;

        let cid_strings: Vec<String> = cids.into_iter().map(|c| c.to_string()).collect();
        delete(RepoBlockSchema::repo_block)
            .filter(RepoBlockSchema::cid.eq_any(cid_strings))
            .execute(conn)?;
        Ok(())
    }

    pub async fn update_root(&self, cid: Cid, rev: String, is_create: Option<bool>) -> Result<()> {
        use crate::schema::registry::repo_root::dsl as RepoRootSchema;
        let conn = &mut establish_connection()?;

        let is_create = is_create.unwrap_or(false);
        if is_create {
            insert_into(RepoRootSchema::repo_root)
                .values((
                    RepoRootSchema::did.eq(&self.did),
                    RepoRootSchema::cid.eq(cid.to_string()),
                    RepoRootSchema::rev.eq(rev),
                    RepoRootSchema::indexedAt.eq(&self.now),
                ))
                .execute(conn)?;
        } else {
            update(RepoRootSchema::repo_root)
                .set((
                    RepoRootSchema::cid.eq(cid.to_string()),
                    RepoRootSchema::rev.eq(rev),
                    RepoRootSchema::indexedAt.eq(&self.now),
                ))
                .execute(conn)?;
        }
        Ok(())
    }

    pub async fn get_root(&self) -> Option<Cid> {
        match self.get_root_detailed().await {
            Ok(root) => Some(root.cid),
            Err(_) => None,
        }
    }

    pub async fn get_root_detailed(&self) -> Result<CidAndRev> {
        use crate::schema::registry::repo_root::dsl as RepoRootSchema;
        let conn = &mut establish_connection()?;

        let res = RepoRootSchema::repo_root
            .filter(RepoRootSchema::did.eq(&self.did))
            .select(models::RepoRoot::as_select())
            .first(conn)?;

        Ok(CidAndRev {
            cid: Cid::from_str(&res.cid)?,
            rev: res.rev,
        })
    }
}