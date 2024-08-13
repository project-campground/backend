use chrono::{DateTime, Utc};
use did_method_plc::{Keypair, DIDPLC};
use didkit::{ssi::did::VerificationMethod, DIDResolver, ResolutionInputMetadata};
use did_web::DIDWeb;

use super::jwt::{JWTAlgorithm, JWT};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
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

#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum TokenError {
    #[error("Invalid token")]
    Invalid,
    #[error("Malformed PLC operation")]
    MalformedPLC,
    #[error("Invalid token type")]
    InvalidTokenType,
}

#[derive(Debug)]
pub struct AuthToken {
    token_type: TokenType,
    token: String,

    issuer: String,
    expires: DateTime<Utc>,
}

impl AuthToken {
    fn type_from_token(jwt: &JWT) -> Result<TokenType, TokenError> {
        if jwt.audience.is_some() {
            return Ok(TokenType::Interservice);
        }
        Ok(TokenType::try_from(jwt.subject.as_ref().unwrap())
            .map_err(|_| TokenError::Invalid)?)
    }

    pub async fn from_token(plc: &DIDPLC, web: &DIDWeb, key: &Keypair, token: &str) -> Result<Self, TokenError> {
        let jwt = JWT::from_str(token).unwrap();
        let token_type = AuthToken::type_from_token(&jwt)?;
        match token_type {
            TokenType::Interservice => {
                let issuer = jwt.issuer.as_str();
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

                jwt.verify(&key).map_err(|_| TokenError::Invalid)?;
                Ok(Self {
                    token_type,
                    issuer: issuer.to_string(),
                    token: token.to_string(),
                    expires: DateTime::from_timestamp(jwt.expiration_time.unwrap(), 0).unwrap(),
                })
            }
            _ => {
                jwt.verify(&key).map_err(|_| TokenError::Invalid)?;
                Ok(Self {
                    token_type,
                    issuer: jwt.issuer.clone(),
                    token: token.to_string(),
                    expires: DateTime::from_timestamp(jwt.expiration_time.unwrap(), 0).unwrap(),
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
        let mut jwt = JWT {
            issuer: did.to_string(),
            expiration_time: Some(Utc::now().timestamp() + token_lifetime),
            subject: Some(token_type.to_string()),
            audience: None,
            algorithm: JWTAlgorithm::default(),
            jwt_type: "JWT".to_owned(),
            not_before: None,
            issued_at: None,
            signature: None,
        };
        let _ = jwt.sign(key);
        let token_str = jwt.to_string();
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
        let mut jwt = JWT {
            issuer: did.to_string(),
            expiration_time: Some(Utc::now().timestamp() + 60),
            subject: None,
            audience: Some(service_did.to_string()),
            algorithm: JWTAlgorithm::default(),
            jwt_type: "JWT".to_owned(),
            not_before: None,
            issued_at: None,
            signature: None,
        };
        let _ = jwt.sign(key);
        let token_str = jwt.to_string();
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
