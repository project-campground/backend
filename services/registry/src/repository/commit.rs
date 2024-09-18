use multihash_codetable::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};
use did_method_plc::Keypair;
use cid::Cid;

use super::{RecordKey, TID};

#[derive(thiserror::Error, Debug, Copy, Clone)]
pub enum Error {
    #[error("Serialization Error")]
    Serialization,
    #[error("Invalid Signature")]
    InvalidSignature,
    #[error("Signing Error")]
    Signing,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsignedCommit {
    pub did: String,
    pub version: u64,
    pub data: Cid,
    pub rev: TID,
    pub prev: Option<Cid>,
}

impl UnsignedCommit {
    pub fn to_signed(self, keypair: &Keypair) -> Result<SignedCommit, Error> {
        let dag = serde_ipld_dagcbor::to_vec(&self).map_err(|_| Error::Serialization)?;
        let hash = sha256::digest(&dag);
        let sig = keypair
            .sign(hex::decode(hash).map_err(|_| Error::Signing)?.as_slice())
            .map_err(|_| Error::Signing)?;
        Ok(SignedCommit {
            unsigned: self,
            sig,
        })
    }
}

impl From<Vec<u8>> for UnsignedCommit {
    fn from(value: Vec<u8>) -> Self {
        serde_ipld_dagcbor::from_slice(&value).unwrap()
    }
}

impl From<&[u8]> for UnsignedCommit {
    fn from(value: &[u8]) -> Self {
        serde_ipld_dagcbor::from_slice(value).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedCommit {
    #[serde(flatten)]
    pub unsigned: UnsignedCommit,
    pub sig: Vec<u8>,
}

impl SignedCommit {
    pub fn verify(&self, keypair: &Keypair) -> Result<bool, Error> {
        let dag = serde_ipld_dagcbor::to_vec(&self.unsigned).map_err(|_| Error::Serialization)?;
        let hash = sha256::digest(&dag);
        Ok(keypair
            .verify(
                hex::decode(hash)
                    .map_err(|_| Error::InvalidSignature)?
                    .as_slice(),
                &self.sig.as_slice(),
            )
            .map_err(|_| Error::InvalidSignature)?)
    }
}

impl From<Vec<u8>> for SignedCommit {
    fn from(value: Vec<u8>) -> Self {
        serde_ipld_dagcbor::from_slice(&value).unwrap()
    }
}

impl From<&[u8]> for SignedCommit {
    fn from(value: &[u8]) -> Self {
        serde_ipld_dagcbor::from_slice(value).unwrap()
    }
}

impl From<SignedCommit> for Cid {
    fn from(value: SignedCommit) -> Self {
        let dag = serde_ipld_dagcbor::to_vec(&value).unwrap();
        let result = Code::Sha2_256.digest(&dag.as_slice());
        Cid::new_v1(0x71, result)
    }
}
