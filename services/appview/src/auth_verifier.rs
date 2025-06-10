use atproto_identity::{key::identify_key, plc, resolve::resolve_subject, storage::DidDocumentStorage, storage_lru::LruDidDocumentStorage, web};
use base64::{engine::general_purpose, Engine};
use reqwest::Client;

use rocket::request::{FromRequest, Outcome, Request};
use atproto_oauth::jwt::{Claims, Header};
use anyhow::{bail, Result};
use rocket::http::Status;
use thiserror::Error;
use rocket::State;

use crate::DNS_RESOLVER;

const BEARER: &str = "Bearer ";

/// JWT authorization extractor that validates tokens against cached DID documents.
///
/// Contains JWT header, validated claims, original token.
pub struct Authorization(pub Header, pub Claims, pub String);

/// JWT authorization extractor that validates tokens against cached DID documents.
/// Does not trigger an unauthorized error on failure.
/// 
/// Contains JWT header, validated claims, original token, and validation status.
pub struct OptionalAuthorization(pub Header, pub Claims, pub String, pub bool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalAuthorization {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = match req.headers().get_one("Authorization") {
            Some(header) => header,
            None => return Outcome::Success(OptionalAuthorization(Header::default(), Claims::default(), "".to_string(), false))
        };
        let token = match auth_header.strip_prefix(BEARER) {
            Some(token) => token.to_string(),
            None => return Outcome::Success(OptionalAuthorization(Header::default(), Claims::default(), "".to_string(), false))
        };

        let http_client = req.guard::<&State<Client>>().await.unwrap();
        let did_document_storage = req.guard::<&State<LruDidDocumentStorage>>().await.unwrap();

        match validate_jwt(&token, &did_document_storage, &*http_client).await {
            Ok((header, claims)) => {
                Outcome::Success(OptionalAuthorization(header, claims, token, true))
            },
            Err(_e) => Outcome::Success(OptionalAuthorization(Header::default(), Claims::default(), "".to_string(), false))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authorization {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = match req.headers().get_one("Authorization") {
            Some(header) => header,
            None => return Outcome::Error((Status::Unauthorized, AuthError::AuthRequired))
        };
        let token = match auth_header.strip_prefix(BEARER) {
            Some(token) => token.to_string(),
            None => return Outcome::Error((Status::Unauthorized, AuthError::AuthRequired))
        };

        let http_client = req.guard::<&State<Client>>().await.unwrap();
        let did_document_storage = req.guard::<&State<LruDidDocumentStorage>>().await.unwrap();

        match validate_jwt(&token, &did_document_storage, &*http_client).await {
            Ok((header, claims)) => {
                Outcome::Success(Authorization(header, claims, token))
            },
            Err(_e) => Outcome::Error((Status::Unauthorized, AuthError::AuthRequired))
        }
    }
}

async fn validate_jwt(
    token: &str,
    storage: &State<LruDidDocumentStorage>,
    http_client: &Client
) -> Result<(Header, Claims)> {
    // Split and decode JWT
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::BadJwt("Invalid JWT".to_string()).into());
    }

    // Decode claims to get issuer
    let encoded_claims = parts[1];
    let claims_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(encoded_claims)
        .map_err(|e| AuthError::BadJwt(e.to_string()))?;

    let claims: Claims = serde_json::from_slice(&claims_bytes)
        .map_err(|e| AuthError::BadJwt(e.to_string()))?;

    // Get issuer from claims
    let iss = claims
        .jose
        .issuer
        .as_ref()
        .ok_or_else(|| AuthError::BadJwt("Missing issuer".to_string()))?;

    // Try to look up DID document from storage
    let mut did_document = storage.get_document_by_did(iss).await?;

    // If not found, try to resolve the subject
    if did_document.is_none() {
        let did = resolve_subject(http_client, &DNS_RESOLVER, iss).await?;
        let document = match *did.split(":").collect::<Vec<&str>>().get(1).unwrap() {
            "plc" => {
                plc::query(http_client, "plc.directory", &did).await?
            },
            "web" => {
                web::query(http_client, &did).await?
            },
            _ => bail!("Unknown DID method")
        };
        did_document = Some(document);
    }

    let did_document = did_document.ok_or_else(|| AuthError::BadJwt("DID document not found".to_string()))?;

    // Extract keys from DID document
    let did_keys = did_document.did_keys();
    if did_keys.is_empty() {
        return Err(AuthError::BadJwt("No keys found in DID document".to_string()).into());
    }

    for key_multibase in did_keys {
        match identify_key(key_multibase) {
            Ok(key_data) => {
                match atproto_oauth::jwt::verify(token, &key_data) {
                    Ok(validated_claims) => {
                        // Decode header for return
                        let encoded_header = parts[0];
                        let header_bytes = general_purpose::URL_SAFE_NO_PAD
                            .decode(encoded_header)
                            .map_err(|e| AuthError::BadJwt(e.to_string()))?;
                        let header: Header = serde_json::from_slice(&header_bytes)
                            .map_err(|e| AuthError::BadJwt(e.to_string()))?;
                        return Ok((header, validated_claims));
                    }
                    Err(_e) => {
                        continue;
                    }
                }
            }
            Err(_e) => {
                continue;
            }
        }
    }
    
    Err(AuthError::AuthRequired.into())
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("BadJwt: `{0}`")]
    BadJwt(String),
    #[error("BadJwtAudience: `{0}`")]
    BadJwtAudience(String),
    #[error("UntrustedIss: `{0}`")]
    UntrustedIss(String),
    #[error("AuthRequired")]
    AuthRequired,
    #[error("AccountNotFound: `{0}`")]
    AccountNotFound(String),
    #[error("AccountTakedown: `{0}`")]
    AccountTakedown(String),
    #[error("AccountDeactivated: `{0}`")]
    AccountDeactivated(String),
}