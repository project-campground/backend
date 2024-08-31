use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use cid::Cid;

use super::{util::to_cid, ReadableBlockstore};

#[derive(thiserror::Error, Debug)]
pub enum MSTError {
    #[error("failed to parse MST: {0}")]
    ParseError(String),
}

#[derive(Serialize, Deserialize)]
pub struct TreeEntry {
    pub p: u8, // prefix count of ascii chars that this key shares with the prev key
    pub k: Vec<u8>, // the rest of the key outside of the shared prefix
    pub v: Cid, // value
    pub t: Option<Cid>, // next subtree (to the right of leaf)
}

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub l: Option<Cid>, // left-most subtree
    pub e: Vec<TreeEntry>, // entries
}

#[derive(Serialize, Deserialize, Eq)]
#[serde(untagged)]
pub enum NodeEntry {
    MST(MST),
    Leaf(Leaf),
}

impl NodeEntry {
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

impl PartialEq for NodeEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NodeEntry::MST(a), NodeEntry::MST(b)) => a == b,
            (NodeEntry::Leaf(a), NodeEntry::Leaf(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Eq)]
pub struct MST {
    pointer: Cid,
    entries: Option<Vec<NodeEntry>>,
    layer: Option<u32>,
    outdated_pointer: Option<bool>,
}

impl MST {
    pub fn new(entries: Vec<NodeEntry>, layer: Option<u32>) -> Result<Self, MSTError> {
        let pointer = match to_cid(&entries) {
            Some(cid) => cid,
            None => return Err(MSTError::ParseError("failed to parse MST".to_string())),
        };
        Ok(Self {
            pointer,
            entries: Some(entries),
            layer,
            outdated_pointer: None,
        })
    }

    pub fn from_data(data: NodeData, layer: Option<u32>) {
        
    }
}

impl PartialEq for MST {
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