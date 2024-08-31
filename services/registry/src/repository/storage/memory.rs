use cid::Cid;

use crate::repository::BlockMap;

pub struct MemoryBlockstore {
    blocks: BlockMap,
    root: Option<Cid>,
    rev: Option<String>,
}

impl Default for MemoryBlockstore {
    fn default() -> Self {
        Self {
            blocks: BlockMap::new(),
            root: None,
            rev: None,
        }
    }
}