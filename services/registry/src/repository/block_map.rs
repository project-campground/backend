use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Add};
use cid::Cid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Serialization failed")]
    Serialization,
}

pub struct GetManyResult {
    pub blocks: BlockMap,
    pub missing: Vec<Cid>,
}

#[derive(Clone, Debug, Eq)]
pub struct BlockMap(HashMap<Cid, Vec<u8>>);

impl BlockMap {
    pub fn new() -> Self {
        BlockMap(HashMap::new())
    }

    pub fn add<T>(&mut self, value: T) -> Result<Cid, Error>
        where T: Serialize + Deserialize<'static>
    {
        let cid = match super::util::to_cid(&value) {
            Some(cid) => cid,
            None => return Err(Error::Serialization),
        };
        let block = match serde_ipld_dagcbor::to_vec(&value) {
            Ok(dag) => dag,
            Err(_) => return Err(Error::Serialization),
        };
        self.set(cid, block);
        Ok(cid)
    }

    pub fn set(&mut self, cid: Cid, block: Vec<u8>) {
        self.0.insert(cid, block);
    }

    pub fn get(&self, cid: &Cid) -> Option<&Vec<u8>> {
        self.0.get(cid)
    }

    pub fn delete(&mut self, cid: &Cid) {
        self.0.remove(cid);
    }

    pub fn get_many(&self, cids: Vec<Cid>) -> GetManyResult {
        let mut blocks = HashMap::new();
        let mut missing = Vec::new();
        for cid in cids {
            if let Some(block) = self.get(&cid) {
                blocks.insert(cid, block.clone());
            } else {
                missing.push(cid);
            }
        }
        GetManyResult {
            blocks: BlockMap::from(blocks),
            missing
        }
    }

    pub fn has(&self, cid: &Cid) -> bool {
        self.0.contains_key(cid)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn for_each<F>(&self, mut f: F)
        where F: FnMut(&Cid, &[u8])
    {
        for (cid, block) in self.0.iter() {
            f(cid, block);
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&Cid, &Vec<u8>)> {
        self.0.iter()
    }

    pub fn cids(&self) -> impl Iterator<Item = &Cid> {
        self.0.keys()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn byte_size(&self) -> usize {
        self.0.values().map(|v| v.len()).sum()
    }
}

impl Add for BlockMap {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut map = self.0.clone();
        for (cid, block) in other.0 {
            map.insert(cid, block);
        }
        BlockMap(map)
    }
}

impl PartialEq for BlockMap {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for entry in self.entries() {
            let other_bytes = other.get(entry.0);
            if other_bytes.is_none() || entry.1 != other_bytes.unwrap() {
                return false;
            }
        }
        true
    }
}

impl From<HashMap<Cid, Vec<u8>>> for BlockMap {
    fn from(map: HashMap<Cid, Vec<u8>>) -> Self {
        BlockMap(map)
    }
}