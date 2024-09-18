use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use super::{
    mst::{Leaf, NodeData, NodeEntry},
    storage::Blockstore,
    MST,
};

const S32_CHAR: &str = "234567abcdefghijklmnopqrstuvwxyz";

pub fn is_s32(s: &str) -> bool {
    // Check if the string decodes into valid s32
    let mut i = 0.0;
    for c in s.chars() {
        let pos = S32_CHAR.find(c);
        if pos == None {
            return false;
        }
        i = i * 32.0 + pos.unwrap() as f64;
    }
    true
}

pub fn s32encode(i: f64) -> String {
    let mut s = "".to_owned();
    let mut i = i;
    while i > 0 as f64 {
        let c = i % 32.0;
        i = (i / 32.0).floor();
        s = S32_CHAR.chars().nth(c as usize).unwrap().to_string() + &s;
    }
    s.to_string()
}

pub fn s32decode(s: String) -> f64 {
    let mut i = 0.0;
    for c in s.chars() {
        i = i * 32.0 + S32_CHAR.find(c).unwrap() as f64;
    }
    i
}

pub fn to_cid<T>(value: &T) -> Option<Cid>
where
    T: Serialize,
{
    let dag = match serde_ipld_dagcbor::to_vec(value) {
        Ok(dag) => dag,
        Err(_) => return None,
    };
    let result = Code::Sha2_256.digest(&dag.as_slice());
    let cid = Cid::new_v1(0x71, result);
    Some(cid)
}

pub fn compute_depth(bytes: &[u8]) -> u8 {
    let hash: [u8; 32] = sha2::Sha256::digest(bytes).into();
    let mut leading_zeroes: f64 = 0.0;
    for i in 0..hash.len() {
        let byte = hash[i];
        if byte < 64 {
            leading_zeroes += 1.0
        }
        if byte < 16 {
            leading_zeroes += 1.0
        }
        if byte < 4 {
            leading_zeroes += 1.0
        }
        if byte == 0 {
            leading_zeroes += 1.0
        } else {
            break;
        }
    }
    leading_zeroes as u8
}

pub fn count_prefix_len(a: &str, b: &str) -> usize {
    for i in 0..a.len() - 1 {
        if a.chars().nth(i) != b.chars().nth(i) {
            return i;
        }
    }
    return 0;
}

pub fn is_valid_mst_key(s: &str) -> bool {
    let split: Vec<&str> = s.split("/").collect();

    s.len() < 256
        && split.len() == 2
        && split[0].len() > 0
        && split[1].len() > 0
        && is_valid_chars(split[0])
        && is_valid_chars(split[1])
}

fn is_valid_chars(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ':')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_depth() {
        assert_eq!(compute_depth(b"2653ae71"), 0);
        assert_eq!(compute_depth(b"blue"), 1);
        assert_eq!(compute_depth(b"app.bsky.feed.post/454397e440ec"), 4);
        assert_eq!(compute_depth(b"app.bsky.feed.post/9adeb165882c"), 8);
    }
}
