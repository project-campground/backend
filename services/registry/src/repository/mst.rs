use std::collections::HashMap;
use std::str::{from_utf8, Utf8Error};

use cid::Cid;
use serde::{Deserialize, Serialize};

use super::util::{count_prefix_len, is_valid_mst_key};
use super::{
    storage::Blockstore,
    util::{compute_depth, to_cid},
    ReadableBlockstore,
};

#[derive(thiserror::Error, Debug)]
pub enum MSTError {
    #[error("Failed to parse MST: {0}")]
    ParseError(String),
    #[error("No entries or CID provided")]
    NoEntries,
}

#[derive(Debug, thiserror::Error)]
pub enum NodeDataError {
    #[error("Invalid UTF-8: {0}")]
    UTF8(#[from] Utf8Error),
    #[error("Not a valid node: two subtrees next to each other")]
    InvalidNode,
    #[error("Not a valid MST key: {0}")]
    InvalidMSTKey(&'static str),
}

pub type SubTreePointer = Option<Cid>;

#[derive(Serialize, Deserialize)]
pub struct TreeEntry {
    pub p: usize,   // prefix count of ascii chars that this key shares with the prev key
    pub k: Vec<u8>, // the rest of the key outside of the shared prefix
    pub v: Cid,     // value
    pub t: SubTreePointer, // next subtree (to the right of leaf)
}

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub l: SubTreePointer, // left-most subtree
    pub e: Vec<TreeEntry>, // entries
}

impl NodeData {
    pub fn deserialize<'a>(
        storage: &'a Blockstore,
        data: NodeData,
    ) -> Result<Vec<NodeEntry>, NodeDataError> {
        let mut entries: Vec<NodeEntry> = vec![];
        if let Some(cid) = data.l {
            entries.push(NodeEntry::MST(MST::load(cid, storage)));
        }
        let mut last_key = "".to_string();
        for i in 0..data.e.len() {
            let entry = &data.e[i];
            let key_str = from_utf8(entry.k.as_slice())?;
            let key = last_key[0..entry.p].to_string() + key_str;
            if !is_valid_mst_key(&key) {
                return Err(NodeDataError::InvalidMSTKey(&key));
            }
            entries.push(NodeEntry::Leaf(Leaf {
                key: key.clone(),
                value: entry.v,
            }));
            last_key = key;
            if let Some(cid) = entry.t {
                entries.push(NodeEntry::MST(MST::load(cid, storage)));
            }
        }
        Ok(entries)
    }

    pub fn serialize(entries: &Vec<NodeEntry>) -> Result<NodeData, NodeDataError> {
        let mut data = NodeData { l: None, e: vec![] };
        let mut i = 0;
        if entries.len() > 0 {
            if entries[0].is_mst() {
                i += 1;
                match entries[0] {
                    NodeEntry::MST(mst) => data.l = Some(mst.pointer),
                    _ => unreachable!(),
                };
            }
            let mut last_key = "";
            while i < entries.len() {
                let leaf = match entries[i] {
                    NodeEntry::Leaf(entry) => entry,
                    _ => return Err(NodeDataError::InvalidNode),
                };
                let next = entries.get(i + 1);
                i += 1;
                let subtree = match next {
                    Some(next) => match next {
                        NodeEntry::MST(mst) => {
                            i += 1;
                            Some(mst.pointer)
                        }
                        _ => None,
                    },
                    None => None,
                };
                if !is_valid_mst_key(&leaf.key) {
                    return Err(NodeDataError::InvalidMSTKey(&leaf.key));
                }
                let prefix_len = count_prefix_len(last_key, &leaf.key);
                data.e.push(TreeEntry {
                    p: prefix_len,
                    k: leaf.key.as_bytes().to_vec(),
                    v: leaf.value.clone(),
                    t: subtree,
                });

                last_key = &leaf.key;
            }
        }
        Ok(data)
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum NodeEntry<'a> {
    MST(MST<'a>),
    Leaf(Leaf),
}

impl<'a> NodeEntry<'a> {
    pub fn is_mst(&self) -> bool {
        match self {
            NodeEntry::MST(_) => true,
            NodeEntry::Leaf(_) => false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            NodeEntry::MST(_) => false,
            NodeEntry::Leaf(_) => true,
        }
    }
}

impl PartialEq for NodeEntry<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NodeEntry::MST(a), NodeEntry::MST(b)) => a == b,
            (NodeEntry::Leaf(a), NodeEntry::Leaf(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Serialize)]
pub struct MST<'a> {
    #[serde(skip)]
    pub storage: &'a Blockstore,
    pub pointer: Cid,
    pub entries: Option<Vec<NodeEntry<'a>>>,

    depth: Option<u8>,
}

impl<'a> MST<'a> {
    pub fn new(
        pointer: Cid,
        storage: &'a Blockstore,
        entries: Vec<NodeEntry>,
    ) -> Result<Self, MSTError> {
        Ok(Self {
            pointer,
            storage,
            entries: Some(entries),
            depth: None,
        })
    }

    pub fn create(storage: &'a Blockstore, entries: Vec<NodeEntry>) -> Result<Self, MSTError> {
        let pointer = match to_cid(&entries) {
            Some(cid) => cid,
            None => return Err(MSTError::ParseError("failed to parse MST".to_string())),
        };
        Ok(Self {
            pointer,
            storage,
            entries: Some(entries),
            depth: None,
        })
    }

    pub fn load(pointer: Cid, storage: &'a Blockstore) -> Self {
        Self {
            pointer,
            storage,
            entries: None,
            depth: None,
        }
    }

    pub async fn entries(&mut self) -> Result<&Vec<NodeEntry>, MSTError> {
        if self.entries.is_none() {
            let data: NodeData = self
                .storage
                .read_obj(self.pointer.clone())
                .await
                .map_err(|_| MSTError::ParseError("Failed to fetch data".to_owned()))?;
            let deserialized = NodeData::deserialize(&self.storage, data)
                .map_err(|_| MSTError::ParseError("Failed to deserialize NodeData".to_owned()))?;
            self.entries = Some(deserialized);
        }
        Ok(self.entries.as_ref().unwrap())
    }

    pub async fn entries_mut(&'a mut self) -> Result<&'a mut Vec<NodeEntry>, MSTError> {
        if self.entries.is_none() {
            let data: NodeData = self
                .storage
                .read_obj(self.pointer.clone())
                .await
                .map_err(|_| MSTError::ParseError("Failed to fetch data".to_owned()))?;
            let deserialized = NodeData::deserialize(&self.storage, data)
                .map_err(|_| MSTError::ParseError("Failed to deserialize NodeData".to_owned()))?;
            self.entries = Some(deserialized);
        }
        Ok(self.entries.as_mut().unwrap())
    }

    /// Computes the depth from the first leaf in the tree.
    /// If none are found, it keeps recursing until it does.
    /// If it still can't find one, then the tree is empty and the node is at depth 0
    pub fn depth(&mut self) -> Option<u8> {
        if let Some(depth) = self.depth {
            return Some(depth);
        }
        match self.entries.as_ref().unwrap().iter().find(|e| e.is_leaf()) {
            Some(entry) => match entry {
                NodeEntry::Leaf(leaf) => {
                    let depth = compute_depth(leaf.key.as_bytes());
                    self.depth = Some(depth);
                    Some(depth)
                }
                NodeEntry::MST(_) => unreachable!(),
            },
            None => match self.entries.as_ref().unwrap().iter().find(|e| e.is_mst()) {
                Some(entry) => match entry {
                    NodeEntry::MST(mst) => mst.depth(),
                    NodeEntry::Leaf(_) => unreachable!(),
                },
                None => Some(0),
            },
        }
    }

    pub fn pointer_outdated(&self) -> Result<bool, MSTError> {
        let pointer = match to_cid(self.entries.as_ref().unwrap()) {
            Some(cid) => cid,
            None => return Err(MSTError::ParseError("failed to parse MST".to_string())),
        };
        Ok(pointer != self.pointer)
    }

    pub async fn get(&'a mut self, key: &str) -> Option<Cid> {
        let index = self.find_leaf_index(key);
        let found = self.index(index).await;
        if let Some(found) = found {
            match found {
                NodeEntry::Leaf(leaf) => {
                    if leaf.key == key {
                        return Some(leaf.value);
                    }
                }
                NodeEntry::MST(_) => (),
            }
        }
        let prev = self.index_mut(index - 1).await;
        if let Some(prev) = prev {
            match prev {
                NodeEntry::MST(mst) => return mst.get(key).await,
                NodeEntry::Leaf(_) => (),
            }
        }
        None
    }

    pub async fn index(&mut self, index: usize) -> Option<&NodeEntry<'_>> {
        match self.entries().await {
            Ok(entries) => entries.get(index),
            Err(_) => None,
        }
    }

    pub async fn index_mut(&'a mut self, index: usize) -> Option<&mut NodeEntry<'_>> {
        match self.entries_mut().await {
            Ok(entries) => entries.get_mut(index),
            Err(_) => None,
        }
    }

    fn find_leaf_index(&self, key: &str) -> usize {
        let entries = self.entries.as_ref().unwrap();
        let index = entries.iter().position(|e| match e {
            NodeEntry::Leaf(entry) => entry.key == key,
            _ => false,
        });
        match index {
            Some(i) => i,
            None => entries.len() - 1,
        }
    }
}

impl PartialEq for MST<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
}

#[derive(Serialize, Deserialize, Eq)]
pub struct Leaf {
    pub key: String,
    pub value: Cid,
}

impl PartialEq for Leaf {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}
