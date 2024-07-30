use did_method_plc::{operation::PLCOperation, BlessedAlgorithm, Keypair, DIDPLC};
use jwt::{Claims, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;
use hmac::Hmac;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType {
    Access,
    Refresh,
    Interservice,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::Access => "access".to_string(),
            TokenType::Refresh => "refresh".to_string(),
            TokenType::Interservice => "interservice".to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Invalid token")]
    Invalid,
    #[error("JWT error: {0}")]
    JWT(#[from] jwt::Error),
    #[error("Malformed PLC operation")]
    MalformedPLC,
}

pub fn extract_token_did(token_str: &str) -> Result<String, TokenError> {
    let unverified: Token<Header, Claims, _> = jwt::Token::parse_unverified(token_str)?;
    let did = unverified.claims().registered.issuer.clone();
    match did {
        Some(did) => Ok(did),
        None => Err(TokenError::Invalid),
    }
}

pub fn create_interservice_token(key: &Keypair, did: &str, service_did: Option<&str>) -> Result<String, TokenError> {
    let alg = BlessedAlgorithm::from(key.codec);
    let mut claims = BTreeMap::new();
    claims.insert("alg", match alg {
        BlessedAlgorithm::K256 => "ES256K",
        BlessedAlgorithm::P256 => "ES256",
    });
    claims.insert("iss", did);
    claims.insert("exp", &"60");
    if let Some(service_did) = service_did {
        claims.insert("aud", service_did);
    }

    let token_str = claims.sign_with_key(key)?;
    Ok(token_str)
}

pub async fn validate_interservice_token(plc: &DIDPLC, service_did: Option<&str>, token_str: &str) -> Result<String, TokenError> {
    let did = extract_token_did(token_str)?;
    let op = plc.get_current_state(&did).await.map_err(|_| TokenError::Invalid)?;
    let key = match op {
        PLCOperation::SignedPLC(op) => Keypair::from_did_key(
            op.unsigned.verification_methods.get("atproto").unwrap().as_str()
        ).map_err(|_| TokenError::Invalid)?,
        PLCOperation::SignedGenesis(op) => Keypair::from_did_key(
            &op.unsigned.signing_key
        ).map_err(|_| TokenError::Invalid)?,
        _ => return Err(TokenError::MalformedPLC),
    };

    let claims: BTreeMap<String, String> = token_str.verify_with_key(&key)?;
    if let Some(service_did) = service_did {
        assert_eq!(claims.get("aud"), Some(&service_did.to_string()));
    }
    Ok(did)
}

pub fn create_token(key: Hmac<Sha256>, did: String, token_type: TokenType) -> Result<String, TokenError> {
    if token_type == TokenType::Interservice {
        panic!("Interservice tokens are created using create_interservice_token")
    }
    let mut claims = BTreeMap::new();
    claims.insert("iss", did);
    claims.insert("sub", token_type.to_string());

    let token_str = claims.sign_with_key(&key)?;
    Ok(token_str)
}

pub fn validate_token(key: Hmac<Sha256>, token_type: TokenType, token_str: String) -> Result<String, TokenError> {
    let claims: BTreeMap<String, String> = token_str.verify_with_key(&key)?;
    match token_type {
        TokenType::Access => {
            assert_eq!(claims.get("sub"), Some(&"access".to_string()));
        }
        TokenType::Refresh => {
            assert_eq!(claims.get("sub"), Some(&"refresh".to_string()));
        }
        TokenType::Interservice => {
            panic!("Interservice tokens are validated using validate_interservice_token")
        }
    }
    let did = claims.get("iss").unwrap().to_string();
    Ok(did)
}
