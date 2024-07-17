use crate::multicodec::MultiEncoded;
use ecdsa::signature::{SignerMut, Verifier};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use serde::{Deserialize, Serialize};

pub enum BlessedAlgorithm {
    P256,
    K256,
}

impl BlessedAlgorithm {
    pub fn codec(&self) -> u64 {
        match self {
            Self::P256 => 0x1200,
            Self::K256 => 0xe7,
        }
    }

    pub fn prefix(&self) -> [u8; 2] {
        match self {
            Self::P256 => [0x80, 0x24],
            Self::K256 => [0xe7, 0x01],
        }
    }
}

impl From<u64> for BlessedAlgorithm {
    fn from(value: u64) -> Self {
        match value {
            0x1200 => Self::P256,
            0xe7 => Self::K256,
            _ => panic!("Unknown algorithm"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Keypair {
    pub public: Option<Vec<u8>>,
    pub secret: Option<Vec<u8>>,
    pub codec: u64,
}

impl Keypair {
    pub fn generate(algo: BlessedAlgorithm) -> Self {
        match algo {
            BlessedAlgorithm::P256 => {
                let sk = p256::ecdsa::SigningKey::random(&mut rand::thread_rng());
                let vk = sk.verifying_key();
                Keypair {
                    public: Some(vk.to_sec1_bytes().to_vec()),
                    secret: Some(sk.to_bytes().to_vec()),
                    codec: algo.codec(),
                }
            },
            BlessedAlgorithm::K256 => {
                let sk = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
                let vk = sk.verifying_key();
                Keypair {
                    public: Some(vk.to_sec1_bytes().to_vec()),
                    secret: Some(sk.to_bytes().to_vec()),
                    codec: algo.codec(),
                }
            },
        }
    }

    pub fn from_value(value: serde_json::Value) -> Result<Self, crate::Error> {
        let keypair: Keypair = serde_json::from_value(value)?;
        Ok(keypair)
    }

    pub fn from_did_key(key: String) -> Result<Self, crate::Error> {
        if !key.starts_with("did:key:") {
            return Err(crate::Error::InvalidKey);
        }
        let key = key.split_at(8).1;
        let (_base, data) = multibase::decode(key)?;
        let decoded_result = MultiEncoded::new(data.as_slice())?;

        Ok(Keypair {
            public: Some(decoded_result.data().to_vec()),
            secret: None,
            codec: decoded_result.codec(),
        })
    }

    pub fn to_did_key(&self) -> Result<String, crate::Error> {
        if self.public.is_none() {
            return Err(crate::Error::InvalidKey);
        }
        let algo = BlessedAlgorithm::from(self.codec);

        match algo {
            BlessedAlgorithm::P256 => {
                let pk = p256::PublicKey::from_sec1_bytes(self.public.as_ref().unwrap().as_slice())?;
                let key = multibase::encode(
                    multibase::Base::Base58Btc,
                    [
                        algo.prefix().to_vec(),
                        pk.to_encoded_point(true).as_bytes().to_vec()
                    ]
                    .concat()
                );
                Ok(format!("did:key:{}", key))
            },
            BlessedAlgorithm::K256 => {
                let pk = k256::PublicKey::from_sec1_bytes(self.public.as_ref().unwrap().as_slice())?;
                let key = multibase::encode(
                    multibase::Base::Base58Btc,
                    [
                        algo.prefix().to_vec(),
                        pk.to_encoded_point(true).as_bytes().to_vec()
                    ]
                    .concat()
                );
                Ok(format!("did:key:{}", key))
            },
        }
    }

    pub fn from_private_key(key: String) -> Result<Self, crate::Error> {
        let (_base, data) = multibase::decode(key)?;
        let decoded_result = MultiEncoded::new(data.as_slice())?;

        match decoded_result.codec() {
            0xe7 => {
                // Secp256k1
                let sk = k256::ecdsa::SigningKey::from_slice(decoded_result.data().into())?;
                let vk = sk.verifying_key();
                Ok(Keypair {
                    public: Some(vk.to_sec1_bytes().to_vec()),
                    secret: Some(decoded_result.data().to_vec()),
                    codec: decoded_result.codec(),
                })
            },
            0x1200 => {
                // P-256
                let sk = p256::ecdsa::SigningKey::from_slice(decoded_result.data().into())?;
                let vk = sk.verifying_key();
                Ok(Keypair {
                    public: Some(vk.to_sec1_bytes().to_vec()),
                    secret: Some(decoded_result.data().to_vec()),
                    codec: decoded_result.codec(),
                })
            },
            _ => Err(crate::Error::InvalidKey),
        }
    }

    pub fn to_private_key(&self) -> Result<String, crate::Error> {
        if self.secret.is_none() {
            return Err(crate::Error::InvalidKey);
        }
        let algo = BlessedAlgorithm::from(self.codec);
        match algo {
            BlessedAlgorithm::P256 => {
                let sk = p256::ecdsa::SigningKey::from_slice(self.secret.clone().unwrap().as_slice())?;
                let key = multibase::encode(
                    multibase::Base::Base58Btc,
                    [
                        algo.prefix().to_vec(),
                        sk.to_bytes().to_vec()
                    ]
                    .concat()
                );
                Ok(key)
            },
            BlessedAlgorithm::K256 => {
                let sk = k256::ecdsa::SigningKey::from_slice(self.secret.clone().unwrap().as_slice())?;
                let key = multibase::encode(
                    multibase::Base::Base58Btc,
                    [
                        algo.prefix().to_vec(),
                        sk.to_bytes().to_vec()
                    ]
                    .concat()
                );
                Ok(key)
            },
        }
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<bool, crate::Error> {
        match self.codec {
            0xe7 => {
                // Secp256k1
                let vk = k256::ecdsa::VerifyingKey::from_sec1_bytes(self.public.as_ref().unwrap().as_slice())?;
                let sig = k256::ecdsa::Signature::from_slice(sig.into())?;
                if vk.verify(&msg, &sig).is_ok() {
                    return Ok(true);
                }
            },
            0x1200 => {
                // P-256
                let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(self.public.as_ref().unwrap().as_slice())?;
                let sig = p256::ecdsa::Signature::from_slice(sig.into())?;
                if vk.verify(&msg, &sig).is_ok() {
                    return Ok(true);
                }
            },
            _ => (),
        }
        Ok(false)
    }

    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, crate::Error> {
        match self.codec {
            0xe7 => {
                // Secp256k1
                let mut sk = k256::ecdsa::SigningKey::from_slice(self.secret.clone().unwrap().as_slice())?;
                let sig: k256::ecdsa::Signature = sk.sign(&msg);
                Ok(sig.to_bytes().to_vec())
            },
            0x1200 => {
                // P-256
                let mut sk = p256::ecdsa::SigningKey::from_slice(self.secret.clone().unwrap().as_slice())?;
                let sig: p256::ecdsa::Signature = sk.sign(&msg);
                Ok(sig.to_bytes().to_vec())
            },
            _ => Err(crate::Error::InvalidKey),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keypair_p256() {
        let keypair = Keypair::generate(BlessedAlgorithm::P256);
        assert!(keypair.public.is_some());
        assert!(keypair.secret.is_some());
        assert_eq!(keypair.codec, 0x1200);
        assert!(keypair.to_did_key().unwrap().starts_with("did:key:zDn"));
    }

    #[test]
    fn test_keypair_k256() {
        let keypair = Keypair::generate(BlessedAlgorithm::K256);
        assert!(keypair.public.is_some());
        assert!(keypair.secret.is_some());
        assert_eq!(keypair.codec, 0xe7);
        assert!(keypair.to_did_key().unwrap().starts_with("did:key:zQ3s"));
    }

    #[test]
    fn test_keypair_from_did_key_p256() {
        let orig_keypair = Keypair::generate(BlessedAlgorithm::P256);
        let did_key = orig_keypair.to_did_key().unwrap();
        let keypair = Keypair::from_did_key(did_key).unwrap();
        assert_eq!(keypair.to_did_key().unwrap(), orig_keypair.to_did_key().unwrap());
        assert_eq!(keypair.codec, orig_keypair.codec);
    }

    #[test]
    fn test_keypair_from_did_key_k256() {
        let orig_keypair = Keypair::generate(BlessedAlgorithm::K256);
        let did_key = orig_keypair.to_did_key().unwrap();
        let keypair = Keypair::from_did_key(did_key).unwrap();
        assert_eq!(keypair.to_did_key().unwrap(), orig_keypair.to_did_key().unwrap());
        assert_eq!(keypair.codec, orig_keypair.codec);
    }

    #[test]
    fn test_keypair_from_private_key_p256() {
        let orig_keypair = Keypair::generate(BlessedAlgorithm::P256);
        let private_key = orig_keypair.to_private_key().unwrap();
        let keypair = Keypair::from_private_key(private_key).unwrap();
        assert_eq!(keypair.to_did_key().unwrap(), orig_keypair.to_did_key().unwrap());
        assert_eq!(keypair.codec, orig_keypair.codec);
    }

    #[test]
    fn test_keypair_from_private_key_k256() {
        let orig_keypair = Keypair::generate(BlessedAlgorithm::K256);
        let private_key = orig_keypair.to_private_key().unwrap();
        let keypair = Keypair::from_private_key(private_key).unwrap();
        assert_eq!(keypair.to_did_key().unwrap(), orig_keypair.to_did_key().unwrap());
        assert_eq!(keypair.codec, orig_keypair.codec);
    }
}