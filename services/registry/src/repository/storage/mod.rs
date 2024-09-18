use cid::Cid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::BufReader, ops::Deref};

use super::BlockMap;

mod memory;

pub use memory::MemoryBlockstore;

pub type RepoRecord = HashMap<String, Cid>;

pub struct GetBlocksResult {
    pub blocks: BlockMap,
    pub missing_blocks: Vec<Cid>,
}

pub struct ReadResult<T>
where
    T: Serialize + Deserialize<'static>,
{
    pub obj: T,
    pub bytes: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
    #[error("Missing block: {0}")]
    MissingBlock(Cid),
    #[error("Serialization error")]
    SerializationError,
}

pub trait ReadableBlockstore {
    async fn get_bytes(&self, cid: Cid) -> Option<Vec<u8>>;
    async fn has(&self, cid: Cid) -> bool;
    async fn get_blocks(&self, cids: &[Cid]) -> GetBlocksResult;
}

pub enum BlockstoreBackend {
    Memory(MemoryBlockstore),
}

pub struct Blockstore {
    backend: BlockstoreBackend,
}

impl Blockstore {
    pub async fn get_bytes(&self, cid: Cid) -> Option<Vec<u8>> {
        match &self.backend {
            BlockstoreBackend::Memory(blockstore) => blockstore.get_bytes(cid).await,
        }
    }

    pub async fn has(&self, cid: Cid) -> bool {
        match &self.backend {
            BlockstoreBackend::Memory(blockstore) => blockstore.has(cid).await,
        }
    }

    pub async fn get_blocks(&self, cids: &[Cid]) -> GetBlocksResult {
        match &self.backend {
            BlockstoreBackend::Memory(blockstore) => blockstore.get_blocks(cids).await,
        }
    }

    pub async fn attempt_read<T>(&self, cid: Cid) -> Option<ReadResult<T>>
    where
        T: Serialize + Deserialize<'static>,
    {
        let bytes = match self.get_bytes(cid).await {
            Some(bytes) => bytes,
            None => return None,
        };
        let buf: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        let obj = serde_ipld_dagcbor::from_slice(buf).unwrap_or(None);
        match obj {
            Some(obj) => Some(ReadResult {
                obj,
                bytes: buf.to_vec(),
            }),
            None => None,
        }
    }

    pub async fn read_obj_and_bytes<T>(&self, cid: Cid) -> Result<ReadResult<T>, ReadError>
    where
        T: Serialize + Deserialize<'static>,
    {
        let read = self.attempt_read::<T>(cid).await;
        if read.is_none() {
            return Err(ReadError::MissingBlock(cid));
        }
        Ok(read.unwrap())
    }

    pub async fn read_obj<T>(&self, cid: Cid) -> Result<T, ReadError>
    where
        T: Serialize + Deserialize<'static>,
    {
        let read = self.read_obj_and_bytes::<T>(cid).await?;
        Ok(read.obj)
    }

    pub async fn read_record(&self, cid: Cid) -> Result<RepoRecord, ReadError> {
        let bytes = match self.get_bytes(cid).await {
            Some(bytes) => bytes,
            None => return Err(ReadError::MissingBlock(cid)),
        };
        let cbor: RepoRecord = match serde_ipld_dagcbor::from_slice(&bytes) {
            Ok(cbor) => cbor,
            Err(_) => return Err(ReadError::SerializationError),
        };
        Ok(cbor)
    }

    pub async fn attempt_read_record(&self, cid: Cid) -> Option<RepoRecord> {
        let record = self.read_record(cid).await;
        if record.is_ok() {
            Some(record.unwrap())
        } else {
            None
        }
    }
}
