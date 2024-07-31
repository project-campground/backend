use chrono::{DateTime, Utc};
use did_method_plc::{operation::PLCOperation, BlessedAlgorithm, Keypair, DIDPLC};
use jwt::{Claims, Header, SignWithKey, Token, VerifyWithKey};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType {
    Access,
    Refresh,
    Interservice,
}

impl From<&String> for TokenType {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "access" => TokenType::Access,
            "refresh" => TokenType::Refresh,
            "interservice" => TokenType::Interservice,
            _ => panic!("Invalid token type"),
        }
    }
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
    #[error("Invalid token type")]
    InvalidTokenType,
}

pub struct AuthToken {
    token_type: TokenType,
    token: String,

    issuer: String,
    expires: DateTime<Utc>,
}

impl AuthToken {
    fn type_from_token(claims: &Claims) -> Result<TokenType, TokenError> {
        let registered = &claims.registered;
        if claims.registered.issuer.is_some() && registered.audience.is_some() {
            return Ok(TokenType::Interservice);
        }
        Ok(TokenType::try_from(registered.subject.as_ref().unwrap())
            .map_err(|_| TokenError::Invalid)?)
    }

    pub async fn from_token(plc: &DIDPLC, key: &Keypair, token: &str) -> Result<Self, TokenError> {
        let unverified: Token<Header, Claims, _> = jwt::Token::parse_unverified(token)?;
        let token_type = AuthToken::type_from_token(unverified.claims())?;
        match token_type {
            TokenType::Interservice => {
                let issuer = unverified.claims().registered.issuer.as_ref().unwrap();
                let (did, _labeler) = issuer.split_once("#").unwrap();
                // TODO: Make this obtain the key from the DID document's #atproto_label verification method for the specified labeler
                let op = plc
                    .get_current_state(&did)
                    .await
                    .map_err(|_| TokenError::Invalid)?;
                let key = match op {
                    PLCOperation::SignedPLC(op) => Keypair::from_did_key(
                        op.unsigned
                            .verification_methods
                            .get("atproto")
                            .unwrap()
                            .as_str(),
                    )
                    .map_err(|_| TokenError::Invalid)?,
                    PLCOperation::SignedGenesis(op) => {
                        Keypair::from_did_key(&op.unsigned.signing_key)
                            .map_err(|_| TokenError::Invalid)?
                    }
                    _ => return Err(TokenError::MalformedPLC),
                };

                let claims: BTreeMap<String, String> = token.verify_with_key(&key)?;
                Ok(Self {
                    token_type,
                    issuer: issuer.to_string(),
                    token: token.to_string(),
                    expires: DateTime::from_timestamp(claims["exp"].parse().unwrap(), 0).unwrap(),
                })
            }
            _ => {
                let claims: BTreeMap<String, String> = token.verify_with_key(key)?;
                Ok(Self {
                    token_type,
                    issuer: claims["iss"].to_string(),
                    token: token.to_string(),
                    expires: DateTime::from_timestamp(claims["exp"].parse().unwrap(), 0).unwrap(),
                })
            }
        }
    }

    pub fn generate_token(
        key: &Keypair,
        did: &str,
        token_type: TokenType,
    ) -> Result<Self, TokenError> {
        if token_type == TokenType::Interservice {
            return Err(TokenError::InvalidTokenType);
        }

        let token_lifetime = match token_type {
            TokenType::Access => 60,
            TokenType::Refresh => 60 * 60 * 24,
            _ => return Err(TokenError::InvalidTokenType),
        };
        let alg = BlessedAlgorithm::from(key.codec);
        let exp = (Utc::now().timestamp() + token_lifetime).to_string();
        let mut claims = BTreeMap::new();
        claims.insert(
            "alg",
            match alg {
                BlessedAlgorithm::K256 => "ES256K",
                BlessedAlgorithm::P256 => "ES256",
            },
        );
        claims.insert("iss", did);
        claims.insert("exp", &exp);

        let token_str = claims.sign_with_key(key)?;
        Ok(Self {
            token_type,
            token: token_str,
            issuer: did.to_string(),
            expires: Utc::now(),
        })
    }

    pub fn generate_interservice(
        key: &Keypair,
        did: &str,
        service_did: &str,
    ) -> Result<Self, TokenError> {
        let alg = BlessedAlgorithm::from(key.codec);
        let exp = (Utc::now().timestamp() + 60).to_string();
        let mut claims = BTreeMap::new();
        claims.insert(
            "alg",
            match alg {
                BlessedAlgorithm::K256 => "ES256K",
                BlessedAlgorithm::P256 => "ES256",
            },
        );
        claims.insert("iss", did);
        claims.insert("exp", &exp);
        claims.insert("aud", service_did);

        let token_str = claims.sign_with_key(key)?;
        Ok(Self {
            token_type: TokenType::Interservice,
            token: token_str,
            issuer: did.to_string(),
            expires: Utc::now(),
        })
    }

    pub fn issuer(&self) -> (&str, Option<&str>) {
        let mut parts = self.issuer.split("#");
        let did = parts.next().unwrap();
        let labeler = parts.next();
        (did, labeler)
    }

    pub fn did(&self) -> &str {
        self.issuer().0
    }

    pub fn labeler(&self) -> Option<&str> {
        self.issuer().1
    }

    pub fn expires(&self) -> DateTime<Utc> {
        self.expires
    }

    pub fn has_expired(&self) -> bool {
        self.expires <= Utc::now()
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}
