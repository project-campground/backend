use cid::Cid;

use crate::repository::BlockMap;

use super::ReadableBlockstore;

pub struct MemoryBlockstore {
    blocks: BlockMap,
    root: Option<Cid>,
    rev: Option<String>,
}

impl ReadableBlockstore for MemoryBlockstore {
    async fn get_bytes(&self, cid: Cid) -> Option<Vec<u8>> {}

    async fn has(&self, cid: Cid) -> bool {}

    async fn get_blocks(&self, cids: &[Cid]) -> GetBlocksResult {}
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
