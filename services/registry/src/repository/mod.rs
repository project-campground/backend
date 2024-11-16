#![allow(dead_code, unused_imports)]
use crate::config::SECRET_CONFIG;
use crate::database::establish_connection;
use crate::repository::blob::BlobReader;
use crate::repository::data_diff::DataDiff;
use crate::repository::mst::MST;
use crate::repository::preference::PreferenceReader;
use crate::repository::record::RecordReader;
use crate::repository::storage::RepoReader;
use rsky_pds::common;
use rsky_pds::common::tid::{Ticker, TID};
use rsky_pds::repo::aws::s3::S3BlobStore;
use rsky_pds::repo::block_map::BlockMap;
use rsky_pds::repo::cid_set::CidSet;
use rsky_pds::repo::error::DataStoreError;
use rsky_pds::repo::parse;
use rsky_pds::repo::types::{
    write_to_op, CollectionContents, Commit, CommitData,
    PreparedCreateOrUpdate, PreparedWrite, RecordCreateOrUpdateOp, RecordWriteEnum,
    RecordWriteOp, RepoContents, RepoRecord, UnsignedCommit, WriteOpAction,
};
use rsky_pds::repo::util;
use anyhow::{anyhow, bail, Result};
use diesel::*;
use futures::stream::{self, StreamExt};
use futures::try_join;
use lexicon_cid::Cid;
use libipld::cbor::DagCborCodec;
use libipld::Ipld as VendorIpld;
use libipld::{Block, DefaultParams};
use secp256k1::{Keypair, Secp256k1, SecretKey};
use serde_cbor::Value as CborValue;
use std::collections::BTreeMap;
use std::str::FromStr;

pub struct CommitRecord {
    collection: String,
    rkey: String,
    cid: Cid,
    record: RepoRecord,
}

#[derive(Debug)]
pub struct Repo {
    storage: RepoReader, // get ipld blocks from db
    data: MST,
    commit: Commit,
    cid: Cid,
}

pub struct ActorStore {
    pub did: String,
    pub storage: RepoReader, // get ipld blocks from db
    pub record: RecordReader,   // get lexicon records from db
    pub blob: BlobReader,       // get blobs
    pub pref: PreferenceReader, // get preferences
}

// Combination of RepoReader/Transactor, BlobReader/Transactor, RepoReader/Transactor
impl ActorStore {
    /// Concrete reader of an individual repo (hence S3BlobStore which takes `did` param)
    pub fn new(did: String, blobstore: S3BlobStore) -> Self {
        ActorStore {
            storage: RepoReader::new(None, did.clone(), None),
            record: RecordReader::new(did.clone()),
            pref: PreferenceReader::new(did.clone()),
            did,
            blob: BlobReader::new(blobstore), // Unlike TS impl, just use blob reader vs generator
        }
    }

    // Transactors
    // -------------------

    // @TODO: Update to use AtUri
    pub async fn create_repo(
        &mut self,
        keypair: Keypair,
        writes: Vec<PreparedCreateOrUpdate>,
    ) -> Result<CommitData> {
        let write_ops = writes
            .clone()
            .into_iter()
            .map(|prepare| {
                let uri_without_prefix = prepare.uri.replace("at://", "");
                let parts = uri_without_prefix.split("/").collect::<Vec<&str>>();
                let collection = *parts.get(0).unwrap_or(&"");
                let rkey = *parts.get(1).unwrap_or(&"");

                RecordCreateOrUpdateOp {
                    action: WriteOpAction::Create,
                    collection: collection.to_owned(),
                    rkey: rkey.to_owned(),
                    record: prepare.record,
                }
            })
            .collect::<Vec<RecordCreateOrUpdateOp>>();
        let commit = Repo::format_init_commit(
            self.storage.clone(),
            self.did.clone(),
            keypair,
            Some(write_ops),
        )?;
        self.storage.apply_commit(commit.clone(), None).await?;
        let writes = writes
            .into_iter()
            .map(|w| PreparedWrite::Create(w))
            .collect::<Vec<PreparedWrite>>();
        self.blob.process_write_blobs(writes).await?;
        Ok(commit)
    }

    pub async fn process_writes(
        &mut self,
        writes: Vec<PreparedWrite>,
        swap_commit_cid: Option<Cid>,
    ) -> Result<CommitData> {
        let commit = self.format_commit(writes.clone(), swap_commit_cid).await?;
        {
            let immutable_borrow = &self;
            // & send to indexing
            immutable_borrow
                .index_writes(writes.clone(), &commit.rev)
                .await?;
        }
        try_join!(
            // persist the commit to repo storage
            self.storage.apply_commit(commit.clone(), None),
            // process blobs
            self.blob.process_write_blobs(writes)
        )?;
        Ok(commit)
    }

    pub async fn format_commit(
        &mut self,
        writes: Vec<PreparedWrite>,
        swap_commit: Option<Cid>,
    ) -> Result<CommitData> {
        let current_root = self.storage.get_root_detailed().await;
        if let Ok(current_root) = current_root {
            if let Some(swap_commit) = swap_commit {
                if !current_root.cid.eq(&swap_commit) {
                    bail!("BadCommitSwapError: {0}", current_root.cid)
                }
            }
            self.storage.cache_rev(current_root.rev).await?;
            let mut new_record_cids: Vec<Cid> = vec![];
            let mut delete_and_update_uris: Vec<String> = vec![];
            for write in &writes {
                match write.clone() {
                    PreparedWrite::Create(c) => new_record_cids.push(c.cid),
                    PreparedWrite::Update(u) => {
                        new_record_cids.push(u.cid);
                        delete_and_update_uris.push(u.uri);
                    }
                    PreparedWrite::Delete(d) => delete_and_update_uris.push(d.uri),
                }
                if write.swap_cid().is_none() {
                    continue;
                }
                let record = self
                    .record
                    .get_record(write.uri(), None, Some(true))
                    .await?;
                let current_record = match record {
                    Some(record) => Some(Cid::from_str(&record.cid)?),
                    None => None,
                };
                match write {
                    // There should be no current record for a create
                    PreparedWrite::Create(_) if write.swap_cid().is_some() => {
                        bail!("BadRecordSwapError: `{0:?}`", current_record)
                    }
                    // There should be a current record for an update
                    PreparedWrite::Update(_) if write.swap_cid().is_none() => {
                        bail!("BadRecordSwapError: `{0:?}`", current_record)
                    }
                    // There should be a current record for a delete
                    PreparedWrite::Delete(_) if write.swap_cid().is_none() => {
                        bail!("BadRecordSwapError: `{0:?}`", current_record)
                    }
                    _ => Ok::<(), anyhow::Error>(()),
                }?;
                match (current_record, write.swap_cid()) {
                    (Some(current_record), Some(swap_cid)) if current_record.eq(swap_cid) => {
                        Ok::<(), anyhow::Error>(())
                    }
                    _ => bail!(
                        "BadRecordSwapError: current record is `{0:?}`",
                        current_record
                    ),
                }?;
            }
            let mut repo = Repo::load(&mut self.storage, Some(current_root.cid)).await?;
            let write_ops: Vec<RecordWriteOp> = writes
                .into_iter()
                .map(|write| write_to_op(write))
                .collect::<Vec<RecordWriteOp>>();
            // @TODO: Use repo signing key global config
            let secp = Secp256k1::new();
            let repo_private_key = SECRET_CONFIG.repo_signing_key.clone();
            let repo_secret_key =
                SecretKey::from_slice(&hex::decode(repo_private_key.as_bytes()).unwrap()).unwrap();
            let repo_signing_key = Keypair::from_secret_key(&secp, &repo_secret_key);

            let mut commit = repo
                .format_commit(RecordWriteEnum::List(write_ops), repo_signing_key)
                .await?;

            // find blocks that would be deleted but are referenced by another record
            let duplicate_record_cids = self
                .get_duplicate_record_cids(commit.removed_cids.to_list(), delete_and_update_uris)
                .await?;
            for cid in duplicate_record_cids {
                commit.removed_cids.delete(cid)
            }

            // find blocks that are relevant to ops but not included in diff
            // (for instance a record that was moved but cid stayed the same)
            let new_record_blocks = commit.new_blocks.get_many(new_record_cids)?;
            if new_record_blocks.missing.len() > 0 {
                let missing_blocks = self.storage.get_blocks(new_record_blocks.missing).await?;
                commit.new_blocks.add_map(missing_blocks.blocks)?;
            }
            Ok(commit)
        } else {
            bail!("No repo root found for `{0}`", self.did)
        }
    }

    pub async fn index_writes(&self, writes: Vec<PreparedWrite>, rev: &String) -> Result<()> {
        let now: &str = &common::now();

        let _ = stream::iter(writes)
            .then(|write| async move {
                Ok::<(), anyhow::Error>(match write {
                    PreparedWrite::Create(write) => {
                        self.record
                            .index_record(
                                write.uri,
                                write.cid,
                                Some(write.record),
                                Some(write.action),
                                rev.clone(),
                                Some(now.to_string()),
                            )
                            .await?
                    }
                    PreparedWrite::Update(write) => {
                        self.record
                            .index_record(
                                write.uri,
                                write.cid,
                                Some(write.record),
                                Some(write.action),
                                rev.clone(),
                                Some(now.to_string()),
                            )
                            .await?
                    }
                    PreparedWrite::Delete(write) => self.record.delete_record(write.uri).await?,
                })
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    pub async fn destroy(&mut self) -> Result<()> {
        use crate::schema::registry::blob::dsl as BlobSchema;
        let conn = &mut establish_connection()?;

        let blob_rows: Vec<String> = BlobSchema::blob
            .filter(BlobSchema::did.eq(&self.did))
            .select(BlobSchema::cid)
            .get_results(conn)?;
        let cids = blob_rows
            .into_iter()
            .map(|row| Ok(Cid::from_str(&row)?))
            .collect::<Result<Vec<Cid>>>()?;
        let _ = stream::iter(cids.chunks(500))
            .then(|chunk| async {
                Ok::<(), anyhow::Error>(self.blob.blobstore.delete_many(chunk.to_vec()).await?)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    // @TODO: Use AtUri
    pub async fn get_duplicate_record_cids(
        &self,
        cids: Vec<Cid>,
        touched_uris: Vec<String>,
    ) -> Result<Vec<Cid>> {
        if touched_uris.len() == 0 || cids.len() == 0 {
            return Ok(vec![]);
        }
        use crate::schema::registry::record::dsl as RecordSchema;
        let conn = &mut establish_connection()?;

        let cid_strs: Vec<String> = cids.into_iter().map(|c| c.to_string()).collect();
        let res: Vec<String> = RecordSchema::record
            .filter(RecordSchema::did.eq(&self.did))
            .filter(RecordSchema::cid.eq_any(cid_strs))
            .filter(RecordSchema::uri.ne_all(touched_uris))
            .select(RecordSchema::cid)
            .get_results(conn)?;
        Ok(res
            .into_iter()
            .map(|row| Cid::from_str(&row).map_err(|error| anyhow::Error::new(error)))
            .collect::<Result<Vec<Cid>>>()?)
    }
}

impl Repo {
    // static
    pub fn new(storage: RepoReader, data: MST, commit: Commit, cid: Cid) -> Self {
        Repo {
            storage: storage.clone(),
            data,
            commit,
            cid,
        }
    }

    // static
    pub async fn load(storage: &mut RepoReader, cid: Option<Cid>) -> Result<Self> {
        let commit_cid = if let Some(cid) = cid {
            Some(cid)
        } else {
            storage.get_root().await
        };
        match commit_cid {
            Some(commit_cid) => {
                let commit_bytes: Vec<u8> = storage.get_bytes(&commit_cid)?;
                let block = Block::<DefaultParams>::new(commit_cid, commit_bytes.clone())?;
                let ipld = block.decode::<DagCborCodec, VendorIpld>()?;
                // Convert Ipld to Commit
                let commit: Commit = match ipld {
                    VendorIpld::Map(m) => Commit {
                        did: m
                            .get("did")
                            .and_then(|v| {
                                if let VendorIpld::String(s) = v {
                                    Some(s)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| anyhow!("Missing or invalid 'did'"))?
                            .clone(),
                        rev: m
                            .get("rev")
                            .and_then(|v| {
                                if let VendorIpld::String(s) = v {
                                    Some(s)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| anyhow!("Missing or invalid 'rev'"))?
                            .clone(),
                        data: m
                            .get("data")
                            .and_then(|v| {
                                if let VendorIpld::Link(cid) = v {
                                    Some(cid)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| anyhow!("Missing or invalid 'data'"))?
                            .clone(),
                        prev: m.get("prev").and_then(|v| {
                            if let VendorIpld::Link(cid) = v {
                                Some(cid.clone())
                            } else {
                                None
                            }
                        }),
                        version: m
                            .get("version")
                            .and_then(|v| {
                                if let VendorIpld::Integer(i) = v {
                                    Some(*i)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| anyhow!("Missing or invalid 'version'"))?
                            as u8,
                        sig: m
                            .get("sig")
                            .and_then(|v| {
                                if let VendorIpld::Bytes(b) = v {
                                    Some(b)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| anyhow!("Missing or invalid 'sig'"))?
                            .clone(),
                    },
                    _ => return Err(anyhow!("Invalid Ipld format for Commit")),
                };
                let data = MST::load(storage.clone(), commit.data, None)?;
                Ok(Repo::new(storage.clone(), data, commit, commit_cid))
            }
            None => bail!("No cid provided and none in storage"),
        }
    }

    pub fn did(&self) -> String {
        self.commit.did.clone()
    }

    pub fn version(self) -> u8 {
        self.commit.version
    }

    pub fn walk_records(&mut self, from: Option<String>) -> impl Iterator<Item = CommitRecord> {
        let mut iter: Vec<CommitRecord> = Vec::new();
        for leaf in self.data.walk_leaves_from(&from.unwrap_or("".to_owned())) {
            let path = util::parse_data_key(&leaf.key).unwrap();
            let record = self.storage.read_record(&leaf.value).unwrap();
            iter.push(CommitRecord {
                collection: path.collection,
                rkey: path.rkey,
                cid: leaf.value,
                record,
            })
        }
        iter.into_iter()
    }

    pub fn get_record(&mut self, collection: String, rkey: String) -> Result<Option<CborValue>> {
        let data_key = format!("{}/{}", collection, rkey);
        let cid = self.data.get(&data_key)?;
        match cid {
            None => Ok(None),
            Some(cid) => Ok(Some(
                self.storage
                    .read_obj(&cid, |obj| matches!(obj, CborValue::Map(_)))?,
            )),
        }
    }

    pub async fn get_content(&mut self) -> Result<RepoContents> {
        let entries = self.data.list(None, None, None)?;
        let cids = entries
            .clone()
            .into_iter()
            .map(|entry| entry.value)
            .collect::<Vec<Cid>>();
        let found = self.storage.get_blocks(cids).await?;
        if found.missing.len() > 0 {
            return Err(anyhow::Error::new(DataStoreError::MissingBlocks(
                "getContents record".to_owned(),
                found.missing,
            )));
        }
        let mut contents: RepoContents = BTreeMap::new();
        for entry in entries {
            let path = util::parse_data_key(&entry.key)?;
            if contents.get(&path.collection).is_none() {
                contents.insert(path.collection.clone(), CollectionContents::new());
            }
            let parsed = parse::get_and_parse_record(&found.blocks, entry.value)?;
            if let Some(collection_contents) = contents.get_mut(&path.collection) {
                collection_contents.insert(path.rkey, parsed.record);
            }
        }
        Ok(contents.to_owned())
    }

    // static
    pub fn format_init_commit(
        storage: RepoReader,
        did: String,
        keypair: Keypair,
        initial_writes: Option<Vec<RecordCreateOrUpdateOp>>,
    ) -> Result<CommitData> {
        let mut new_blocks = BlockMap::new();
        let mut data = MST::create(storage, None, None)?;
        for record in initial_writes.unwrap_or(Vec::new()) {
            let cid = new_blocks.add(record.record)?;
            let data_key = util::format_data_key(record.collection, record.rkey);
            data = data.add(&data_key, cid, None)?;
        }
        let data_cid: Cid = data.get_pointer()?;
        let diff = DataDiff::of(&mut data, None)?;
        new_blocks.add_map(diff.new_mst_blocks)?;
        let rev = Ticker::new().next(None);
        let commit = util::sign_commit(
            UnsignedCommit {
                did,
                version: 3,
                rev: rev.clone().to_string(),
                prev: None, // added for backwards compatibility with v2
                data: data_cid,
            },
            keypair,
        )?;
        let commit_cid = new_blocks.add(commit)?;
        Ok(CommitData {
            cid: commit_cid,
            rev: rev.clone().to_string(),
            since: None,
            prev: None,
            new_blocks,
            removed_cids: diff.removed_cids,
        })
    }

    // static
    pub async fn create_from_commit(
        storage: &mut RepoReader,
        commit: CommitData,
    ) -> Result<Self> {
        storage.apply_commit(commit.clone(), None).await?;
        Repo::load(storage, Some(commit.cid)).await
    }

    // static
    pub async fn create(
        mut storage: RepoReader,
        did: String,
        keypair: Keypair,
        initial_writes: Option<Vec<RecordCreateOrUpdateOp>>,
    ) -> Result<Self> {
        let commit = Repo::format_init_commit(storage.clone(), did, keypair, initial_writes)?;
        Repo::create_from_commit(&mut storage, commit).await
    }

    pub async fn format_commit(
        &mut self,
        to_write: RecordWriteEnum,
        keypair: Keypair,
    ) -> Result<CommitData> {
        let writes = match to_write {
            RecordWriteEnum::List(to_write) => to_write,
            RecordWriteEnum::Single(to_write) => vec![to_write],
        };
        let mut leaves = BlockMap::new();

        let mut data = self.data.clone();
        for write in writes {
            match write {
                RecordWriteOp::Create(write) => {
                    let cid = leaves.add(write.record)?;
                    let data_key = util::format_data_key(write.collection, write.rkey);
                    data = data.add(&data_key, cid, None)?;
                }
                RecordWriteOp::Update(write) => {
                    let cid = leaves.add(write.record)?;
                    let data_key = util::format_data_key(write.collection, write.rkey);
                    data = data.update(&data_key, cid)?;
                }
                RecordWriteOp::Delete(write) => {
                    let data_key = util::format_data_key(write.collection, write.rkey);
                    data = data.delete(&data_key)?;
                }
            }
        }

        let data_cid = data.get_pointer()?;
        let diff = DataDiff::of(&mut data, Some(&mut self.data.clone()))?;

        let mut new_blocks = diff.new_mst_blocks;
        let mut removed_cids = diff.removed_cids;

        let added_leaves = leaves.get_many(diff.new_leaf_cids.to_list())?;
        if added_leaves.missing.len() > 0 {
            bail!("Missing leaf blocks: {:?}", added_leaves.missing);
        }
        new_blocks.add_map(added_leaves.blocks)?;

        let rev = Ticker::new().next(Some(TID::new(self.commit.rev.clone())?));

        let commit = util::sign_commit(
            UnsignedCommit {
                did: self.did(),
                version: 3,
                rev: rev.clone().to_string(),
                prev: None, // added for backwards compatibility with v2
                data: data_cid,
            },
            keypair,
        )?;
        let commit_cid = new_blocks.add(commit)?;

        // ensure the commit cid actually changed
        if commit_cid.eq(&self.cid) {
            new_blocks.delete(commit_cid)?;
        } else {
            removed_cids.add(self.cid);
        }

        Ok(CommitData {
            cid: commit_cid,
            rev: rev.clone().to_string(),
            since: Some(self.commit.rev.clone()),
            prev: Some(self.cid),
            new_blocks,
            removed_cids,
        })
    }

    pub async fn apply_commit(&mut self, commit_data: CommitData) -> Result<Self> {
        let commit_data_cid = commit_data.cid.clone();
        self.storage.apply_commit(commit_data, None).await?;
        Repo::load(&mut self.storage, Some(commit_data_cid)).await
    }

    pub async fn apply_writes(
        &mut self,
        to_write: RecordWriteEnum,
        keypair: Keypair,
    ) -> Result<Self> {
        let commit = self.format_commit(to_write, keypair).await?;
        self.apply_commit(commit).await
    }

    pub fn format_resign_commit(&self, rev: String, keypair: Keypair) -> Result<CommitData> {
        let commit = util::sign_commit(
            UnsignedCommit {
                did: self.did(),
                version: 3,
                rev: rev.clone(),
                prev: None, // added for backwards compatibility with v2
                data: self.commit.data,
            },
            keypair,
        )?;
        let mut new_blocks = BlockMap::new();
        let commit_cid = new_blocks.add(commit)?;
        Ok(CommitData {
            cid: commit_cid,
            rev,
            since: None,
            prev: None,
            new_blocks,
            removed_cids: CidSet::new(Some(vec![self.cid])),
        })
    }

    pub async fn resign_commit(&mut self, rev: String, keypair: Keypair) -> Result<Self> {
        let formatted = self.format_resign_commit(rev, keypair)?;
        self.apply_commit(formatted).await
    }
}

pub mod preference;
pub mod data_diff;
pub mod storage;
pub mod record;
pub mod sync;
pub mod blob;
pub mod mst;