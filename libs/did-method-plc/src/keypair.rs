use ecdsa::signature::{Verifier, Signer};
use base64::Engine;

#[derive(Debug, thiserror::Error)]
pub enum KeypairError {
    #[error("Missing private key")]
    MissingPrivateKey,
}

pub trait Keypair {
    fn public_key(&self) -> String;
    fn private_key(&self) -> Option<String>;

    fn sign(&mut self, msg: &[u8]) -> Result<String, KeypairError>;
    fn verify(&self, msg: &[u8], sig: String) -> bool;
}

pub struct P256Keypair {
    signing_key: Option<p256::ecdsa::SigningKey>,
    verifying_key: p256::ecdsa::VerifyingKey
}

impl P256Keypair {
    pub fn generate() -> Self {
        let sk = p256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let pk = p256::ecdsa::VerifyingKey::from(&sk);

        Self {
            signing_key: Some(sk),
            verifying_key: pk,
        }
    }

    pub fn from_private_key(sk: String) -> Self {     
        let binding = base64::engine::general_purpose::STANDARD.decode(sk).unwrap();
        let sk = binding.as_slice();
        let sk = p256::ecdsa::SigningKey::from_bytes(sk.into()).unwrap();
        Self { signing_key: Some(sk.clone()), verifying_key: p256::ecdsa::VerifyingKey::from(&sk) }
    }

    pub fn from_public_key(pk: String) -> Self {
        let binding = base64::engine::general_purpose::STANDARD.decode(pk).unwrap();
        let pk = binding.as_slice();
        let pk = p256::ecdsa::VerifyingKey::from_sec1_bytes(pk.into()).unwrap();
        Self { signing_key: None, verifying_key: pk }
    }
}

impl Keypair for P256Keypair {
    fn public_key(&self) -> String {
        let pk = self.verifying_key.to_encoded_point(true);
        base64::engine::general_purpose::STANDARD.encode(pk.to_bytes().as_ref())
    }

    fn private_key(&self) -> Option<String> {
        if self.signing_key.is_none() {
            return None;
        }

        let sk = self.signing_key.as_ref().unwrap().to_bytes();
        Some(base64::engine::general_purpose::STANDARD.encode(sk))
    }

    fn sign(&mut self, msg: &[u8]) -> Result<String, KeypairError> {
        if self.signing_key.is_none() {
            return Err(KeypairError::MissingPrivateKey);
        }

        let sig: p256::ecdsa::Signature = self.signing_key.as_ref().unwrap().sign(msg);
        let r = sig.r().to_bytes();
        let s = sig.s().to_bytes();

        let mut bytes = [0u8; 64];
        bytes[32 - r.len()..32].copy_from_slice(&r);
        bytes[64 - s.len()..64].copy_from_slice(&s);

        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    }
    
    fn verify(&self, msg: &[u8], sig: String) -> bool {
        let binding = base64::engine::general_purpose::STANDARD.decode(sig).unwrap();
        let sig = binding.as_slice();
        let sig = p256::ecdsa::Signature::from_bytes(sig.into()).unwrap();
        let public_key = p256::ecdsa::VerifyingKey::from_sec1_bytes(self.verifying_key.to_sec1_bytes().as_ref()).unwrap();
        public_key.verify(msg, &sig).is_ok()
    }
}

pub struct K256Keypair {
    signing_key: Option<k256::ecdsa::SigningKey>,
    verifying_key: k256::ecdsa::VerifyingKey
}

impl K256Keypair {
    pub fn generate() -> Self { 
        let sk = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let pk = k256::ecdsa::VerifyingKey::from(&sk);

        Self {
            signing_key: Some(sk),
            verifying_key: pk,
        }
    }

    pub fn from_private_key(sk: String) -> Self {        
        let binding = base64::engine::general_purpose::STANDARD.decode(sk).unwrap();
        let sk = binding.as_slice();
        let sk = k256::ecdsa::SigningKey::from_bytes(sk.into()).unwrap();
        Self { signing_key: Some(sk.clone()), verifying_key: k256::ecdsa::VerifyingKey::from(&sk) }
    }

    pub fn from_public_key(pk: String) -> Self { 
        let binding = base64::engine::general_purpose::STANDARD.decode(pk).unwrap();
        let pk = binding.as_slice();
        let pk = k256::ecdsa::VerifyingKey::from_sec1_bytes(pk.into()).unwrap();
        Self { signing_key: None, verifying_key: pk }
    }
}

impl Keypair for K256Keypair {
    fn public_key(&self) -> String {
        let pk = self.verifying_key.to_encoded_point(true);
        base64::engine::general_purpose::STANDARD.encode(pk.to_bytes().as_ref())
    }

    fn private_key(&self) -> Option<String> {
        if self.signing_key.is_none() {
            return None;
        }

        let sk = self.signing_key.as_ref().unwrap().to_bytes();
        Some(base64::engine::general_purpose::STANDARD.encode(sk))
    }

    fn sign(&mut self, msg: &[u8]) -> Result<String, KeypairError> {
        if self.signing_key.is_none() {
            return Err(KeypairError::MissingPrivateKey);
        }

        let sig: k256::ecdsa::Signature = self.signing_key.as_ref().unwrap().sign(msg);
        let r = sig.r().to_bytes();
        let s = sig.s().to_bytes();

        let mut bytes = [0u8; 64];
        bytes[32 - r.len()..32].copy_from_slice(&r);
        bytes[64 - s.len()..64].copy_from_slice(&s);
        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    }
    
    fn verify(&self, msg: &[u8], sig: String) -> bool {        
        let binding = base64::engine::general_purpose::STANDARD.decode(sig).unwrap();
        let sig = binding.as_slice();
        let sig = k256::ecdsa::Signature::from_bytes(sig.into()).unwrap();
        let public_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(self.verifying_key.to_sec1_bytes().as_ref()).unwrap();
        public_key.verify(msg, &sig).is_ok()
    }
}