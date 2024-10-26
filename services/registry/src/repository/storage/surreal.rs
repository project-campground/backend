//! NOTE: This has been abandoned for now as SurrealDB is too unstable
use std::str::FromStr;
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use futures::try_join;
use rsky_pds::car::read_car_bytes;
use rsky_pds::repo::block_map::{BlockMap, BlocksAndMissing};
use rsky_pds::common;
use lexicon_cid::Cid;
use rsky_pds::repo::cid_set::CidSet;
use rsky_pds::repo::types::{CommitData, RepoRecord};
use rsky_pds::storage::ObjAndBytes;
use surrealdb::engine::any::Any;
use rsky_pds::repo::util::cbor_to_lex_record;
use rsky_pds::repo::parse;
use serde_cbor::Value as CborValue;
use surrealdb::sql::Thing;
use serde::{Serialize, Deserialize};
use surrealdb::Bytes;
use crate::database::{RepoBlock, RepoRoot};
use rsky_pds::storage::RepoRootError::RepoRootNotFoundError;

use super::CidAndRev;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartialRepoBlock {
    pub id: Thing,
    cid: String,
    content: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SurrealRepoReader {
    pub connection: surrealdb::Surreal<Any>,
    pub cache: BlockMap,
    pub blocks: BlockMap,
    pub root: Option<Cid>,
    pub rev: Option<String>,
    pub now: DateTime<Utc>,
    pub did: String,
}

impl SurrealRepoReader {
    pub fn new(conn: surrealdb::Surreal<Any>, blocks: Option<BlockMap>, did: String, now: Option<DateTime<Utc>>) -> Self {
        let now = now.unwrap_or_else(|| common::now().parse::<DateTime<Utc>>().unwrap());
        let mut this = SurrealRepoReader {
            connection: conn,
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
        let conn = &self.connection;
        let cached = self.cache.get_many(cids)?;

        if cached.missing.len() < 1 {
            return Ok(cached);
        }
        let mut missing = CidSet::new(Some(cached.missing.clone()));
        let missing_strings: Vec<String> =
            cached.missing.into_iter().map(|c| c.to_string()).collect();

        let mut blocks = BlockMap::new();

        let mut chunks = missing_strings
            .chunks(500)
            .into_iter()
            .map(|c| c.to_owned());
        
        for _ in 1..chunks.len() {
            let batch = chunks.next();
            if let Some(batch) = batch {
                let rows = conn.query("SELECT cid, content FROM repo_block WHERE cid IN $batch && did == $did")
                    .bind(("batch", batch))
                    .bind(("did", self.did.clone()))
                    .await?
                    .take::<Vec<PartialRepoBlock>>(0)?
                    .into_iter();

                for row in rows {
                    let cid = Cid::from_str(&row.cid)?;
                    blocks.set(cid, row.content);
                    missing.delete(cid);
                }
            }
        }

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
                    let res = self.get_block_range(&since, &cursor).await?;
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

    pub async fn get_block_range(
        &self,
        since: &Option<String>,
        cursor: &Option<CidAndRev>,
    ) -> Result<Vec<RepoBlock>> {
        let conn = &self.connection;
        if since.is_some() && cursor.is_some() {
            let since = since.clone().unwrap();
            let cursor = cursor.as_ref().clone().unwrap();

            let result: Vec<RepoBlock> = conn.query("SELECT * FROM repo_block WHERE did == $did && (repoRev, cid) < ($repoRev, $cid) && repoRev > $since ORDER BY repoRev DESC, cid DESC LIMIT 500")
                .bind(("did", self.did.clone()))
                .bind(("repoRev", cursor.rev.clone()))
                .bind(("cid", cursor.cid.to_string()))
                .bind(("since", since))
                .await?
                .take(0)?;
            Ok(result)
        } else if let Some(since) = since {
            let result: Vec<RepoBlock> = conn.query("SELECT * FROM repo_block WHERE did == $did && repoRev > $repoRev ORDER BY repoRev DESC, cid DESC LIMIT 500")
                .bind(("did", self.did.clone()))
                .bind(("repoRev", since.clone()))
                .await?
                .take(0)?;
            Ok(result)
        } else if let Some(cursor) = cursor {
            let result: Vec<RepoBlock> = conn.query("SELECT * FROM repo_block WHERE did == $did && (repoRev, cid) < ($repoRev, $cid) ORDER BY repoRev DESC, cid DESC LIMIT 500")
                .bind(("did", self.did.clone()))
                .bind(("repoRev", cursor.rev.clone()))
                .bind(("cid", cursor.cid.to_string()))
                .await?
                .take(0)?;
            Ok(result)
        } else {
            let result: Vec<RepoBlock> = conn.query("SELECT * FROM repo_block WHERE did == $did ORDER BY repoRev DESC, cid DESC LIMIT 500")
                .bind(("did", self.did.clone()))
                .await?
                .take(0)?;
            Ok(result)
        }
    }

    pub async fn get_bytes(&mut self, cid: &Cid) -> Result<Vec<u8>> {
        let conn = &self.connection;

        println!("Getting cached results for CID");
        let cached = self.cache.get(*cid);
        if let Some(cached_result) = cached {
            return Ok(cached_result.clone());
        }

        println!("Getting results from DB");
        println!("{:?}", conn);
        let mut query = conn.query("SELECT content FROM repo_block WHERE cid == $cid && did == $did LIMIT 1;");
        println!("Created query");
        query = query.bind(("cid", cid.to_string()));
        println!("Bound CID");
        query = query.bind(("did", self.did.clone()));
        println!("Bound DID");
        let mut response = query.await?;
        println!("Got response");
        let result: Option<Bytes> = response.take(0)?;
        // let result: Option<Bytes> = conn.query("SELECT content FROM ONLY repo_block WHERE cid == $cid && did == $did LIMIT 1;")
        //     .bind(("cid", cid.to_string()))
        //     .bind(("did", self.did.clone()))
        //     .await?
        //     .take(0)?;
        println!("Got results from DB");
        if let Some(result) = result {
            self.cache.set(*cid, result.clone().to_vec());
            return Ok(result.to_vec());
        }
        bail!("No results found for CID: {}", cid);
    }

    pub async fn count_blocks(&self) -> Result<i64> {
        let conn = &self.connection;
        let res: Option<usize> = conn.query("SELECT count() FROM repo_block WHERE did == $did")
            .bind(("did", self.did.clone()))
            .await?
            .take(0)?;
        Ok(i64::try_from(res.unwrap_or(0)).unwrap())
    }

    pub async fn has(&mut self, cid: Cid) -> Result<bool> {
        let got = self.get_bytes(&cid).await;
        match got {
            Ok(got) => Ok(!got.is_empty()),
            Err(_) => Ok(false),
        }
    }

    pub async fn attempt_read(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        let bytes = self.get_bytes(cid).await?;
        Ok(parse::parse_obj_by_kind(bytes, *cid, check)?)
    }

    pub async fn read_obj_and_bytes(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<ObjAndBytes> {
        let read = self.attempt_read(cid, check).await?;
        Ok(read)
    }

    pub async fn read_obj(
        &mut self,
        cid: &Cid,
        check: impl Fn(&'_ CborValue) -> bool,
    ) -> Result<CborValue> {
        let obj = self.read_obj_and_bytes(cid, check).await?;
        Ok(obj.obj)
    }

    pub async fn read_record(&mut self, cid: &Cid) -> Result<RepoRecord> {
        let bytes = self.get_bytes(cid).await?;
        Ok(cbor_to_lex_record(bytes)?)
    }

    // Transactors
    // -------------------

    /// Proactively cache all blocks from a particular commit (to prevent multiple roundtrips)
    pub async fn cache_rev(&mut self, rev: String) -> Result<()> {
        let conn = &self.connection;

        let res: Vec<PartialRepoBlock> = conn.query("SELECT cid, content FROM repo_block WHERE repoRev == $repoRev && did == $did LIMIT 15")
            .bind(("repoRev", rev))
            .bind(("did", self.did.clone()))
            .await?
            .take(0)?;
        for row in res {
            self.cache.set(Cid::from_str(&row.cid)?, row.content);
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
        let conn = &self.connection;

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
        
        let chunks = blocks.chunks(50);
        for chunk in chunks {
            let _: Vec<RepoBlock> = conn.insert("repo_block").content(chunk.to_vec()).await?;
        }
        Ok(())
    }

    pub async fn delete_many(&self, cids: Vec<Cid>) -> Result<()> {
        if cids.len() < 1 {
            return Ok(());
        }
        let conn = &self.connection;

        let cid_strings: Vec<String> = cids.into_iter().map(|c| c.to_string()).collect();
        conn.query("DELETE FROM repo_block WHERE cid IN $cids && did == $did")
            .bind(("cids", cid_strings))
            .bind(("did", self.did.clone()))
            .await?;
        Ok(())
    }

    pub async fn update_root(&self, cid: Cid, rev: String, is_create: Option<bool>) -> Result<()> {
        let conn = &self.connection;

        let is_create = is_create.unwrap_or(false);
        if is_create {
            let _: Option<RepoRoot> = conn.insert(("repo_root", self.did.clone()))
                .content(RepoRoot {
                    id: Thing::from(("repo_root", self.did.clone().as_str())),
                    cid: cid.to_string(),
                    indexed_at: self.now.clone(),
                    rev,
                })
                .await?;
        } else {
            let _: Option<RepoRoot> = conn.update(("repo_root", self.did.clone()))
                .content(RepoRoot {
                    id: Thing::from(("repo_root", self.did.clone().as_str())),
                    cid: cid.to_string(),
                    indexed_at: self.now.clone(),
                    rev,
                })
                .await?;
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
        let conn = &self.connection;

        let res: Option<RepoRoot> = conn.query("SELECT * FROM ONLY type::thing(\"repo_root\", $did)")
            .bind(("did", self.did.clone()))
            .await?
            .take(0)?;
        
        match res {
            Some(res) => Ok(CidAndRev {
                cid: Cid::from_str(&res.cid)?,
                rev: res.rev,
            }),
            None => Err(anyhow::anyhow!("No root found")),
        }
    }
}
