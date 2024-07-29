// TODO: Create guards for authenticated lexicons
// TODO: Create guards for admin-authenticated lexicons
// TODO: Create guards for inter-service authenticated lexicons

use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;
use hmac::Hmac;

pub enum TokenType {
    Access,
    Refresh,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::Access => "access".to_string(),
            TokenType::Refresh => "refresh".to_string(),
        }
    }
}

pub fn create_token(key: Hmac<Sha256>, did: String, token_type: TokenType) -> Result<String, jwt::Error> {
    let mut claims = BTreeMap::new();
    claims.insert("did", did);
    claims.insert("sub", token_type.to_string());

    let token_str = claims.sign_with_key(&key)?;
    Ok(token_str)
}

pub fn validate_token(key: Hmac<Sha256>, token_type: TokenType, token_str: String) -> Result<String, jwt::Error> {
    let claims: BTreeMap<String, String> = token_str.verify_with_key(&key)?;
    match token_type {
        TokenType::Access => {
            assert_eq!(claims.get("sub"), Some(&"access".to_string()));
        }
        TokenType::Refresh => {
            assert_eq!(claims.get("sub"), Some(&"refresh".to_string()));
        }
    }
    let did = claims.get("did").unwrap().to_string();
    Ok(did)
}
