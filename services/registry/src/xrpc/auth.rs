use chrono::{DateTime, Utc};
use did_method_plc::{BlessedAlgorithm, Keypair, DIDPLC};
use didkit::{ssi::did::VerificationMethod, DIDResolver, ResolutionInputMetadata};
use jwt::{Claims, Header, SignWithKey, Token, VerifyWithKey};
use std::collections::BTreeMap;
use did_web::DIDWeb;

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

    pub async fn from_token(plc: &DIDPLC, web: &DIDWeb, key: &Keypair, token: &str) -> Result<Self, TokenError> {
        let unverified: Token<Header, Claims, _> = jwt::Token::parse_unverified(token)?;
        let token_type = AuthToken::type_from_token(unverified.claims())?;
        match token_type {
            TokenType::Interservice => {
                let issuer = unverified.claims().registered.issuer.as_ref().unwrap();
                let mut parts = issuer.split("#");
                let did = parts.next().unwrap();
                let service = parts.next();
                
                // TODO: Cache DID resolutions once caching has been implemented
                let (_res_metadata, document, _) = 
                    match did.split(":").collect::<Vec<&str>>()[1] {
                        "plc" => plc.resolve(did, &ResolutionInputMetadata::default()).await,
                        "web" => web.resolve(did, &ResolutionInputMetadata::default()).await,
                        _ => return Err(TokenError::Invalid),
                    };
                if document.is_none() {
                    return Err(TokenError::Invalid);
                }
                let document = document.unwrap();
                let key = match service {
                    Some(service) => {
                        let labeler = match service {
                            "atproto_labeler" => "atproto_label",
                            s => s,
                        };
                        let method = document
                            .verification_method
                            .ok_or(TokenError::Invalid)?
                            .into_iter()
                            .find(|method|
                                match method {
                                    VerificationMethod::Map(map) => map.id == format!("#{0}", labeler),
                                    _ => false,
                                }
                            )
                            .ok_or(TokenError::Invalid)?;
                        let key_multibase = match method {
                            VerificationMethod::Map(map) => map.property_set
                                .as_ref()
                                .ok_or(TokenError::Invalid)?
                                .get("publicKeyMultibase")
                                .ok_or(TokenError::Invalid)?
                                .as_str()
                                .ok_or(TokenError::Invalid)?
                                .to_string(),
                            _ => return Err(TokenError::Invalid),
                        };
                        Keypair::from_did_key(&format!("did:key:{0}", key_multibase)).map_err(|_| TokenError::Invalid)?
                    },
                    None => {
                        let method = document
                            .verification_method
                            .ok_or(TokenError::Invalid)?
                            .into_iter()
                            .find(|method|
                                match method {
                                    VerificationMethod::Map(map) => map.id == "#atproto",
                                    _ => false,
                                }
                            )
                            .ok_or(TokenError::Invalid)?;
                        let key_multibase = match method {
                            VerificationMethod::Map(map) => map.property_set
                                .as_ref()
                                .ok_or(TokenError::Invalid)?
                                .get("publicKeyMultibase")
                                .ok_or(TokenError::Invalid)?
                                .as_str()
                                .ok_or(TokenError::Invalid)?
                                .to_string(),
                            _ => return Err(TokenError::Invalid),
                        };
                        Keypair::from_did_key(&format!("did:key:{0}", key_multibase)).map_err(|_| TokenError::Invalid)?
                    }
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
