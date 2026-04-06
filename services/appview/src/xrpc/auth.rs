#![
    allow(dead_code)
]

use atproto_identity::{key::identify_key, plc, resolve::resolve_subject, storage::DidDocumentStorage, storage_lru::LruDidDocumentStorage, web};
use base64::{engine::general_purpose, Engine};
use reqwest::Client;

use rocket::request::{FromRequest, Outcome, Request};
use atproto_oauth::jwt::{Claims, Header};
use anyhow::Result;
use rocket::http::Status;
use thiserror::Error;
use rocket::State;

use crate::DNS_RESOLVER;

const BEARER: &str = "Bearer ";

/// JWT authorization extractor that validates tokens against cached DID documents.
///
/// Contains JWT header, validated claims, original token.
pub struct Authorization<'a> {
    pub header: Header,
    pub claims: Claims,
    pub token: String,
    pub actor_did: String,
    pub client: &'a State<Client>,
    pub did_document_storage: &'a State<LruDidDocumentStorage>,
}

/// JWT authorization extractor that validates tokens against cached DID documents.
/// Does not trigger an unauthorized error on failure.
/// 
/// Contains JWT header, validated claims, original token, and validation status.
pub enum OptionalAuthorization<'a> {
    Unauthorized,
    Authorized(Authorization<'a>),
}

#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for OptionalAuthorization<'a> where 'r: 'a {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.guard::<Authorization>().await {
            Outcome::Success(value) =>
                Outcome::Success(OptionalAuthorization::Authorized(value)),
            Outcome::Forward(value) =>
                Outcome::Forward(value),
            Outcome::Error(_value) =>
                Outcome::Success(OptionalAuthorization::Unauthorized),
        }
    }
}

#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for Authorization<'a> where 'r: 'a {
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
                Outcome::Success(Authorization { client: http_client, did_document_storage, header, claims: claims.clone(), token, actor_did: claims.jose.issuer.unwrap() })
            },
            Err(_e) => Outcome::Error((Status::Unauthorized, AuthError::AuthRequired))
        }
    }
}

pub async fn validate_jwt(
    token: &str,
    storage: &State<LruDidDocumentStorage>,
    http_client: &Client
) -> Result<(Header, Claims)> {
    // Split and decode JWT
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::InvalidJWT.into());
    }

    // Decode claims to get issuer
    let encoded_claims = parts[1];
    let claims_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(encoded_claims)
        .map_err(|_| AuthError::InvalidClaims)?;

    let claims: Claims = serde_json::from_slice(&claims_bytes)
        .map_err(|_| AuthError::InvalidClaims)?;

    // Get issuer from claims
    let iss = claims
        .jose
        .issuer
        .as_ref()
        .ok_or_else(|| AuthError::MissingIssuer)?;

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
            _ => return Err(AuthError::InvalidDIDMethod(did).into()),
        };
        did_document = Some(document);
    }

    let did_document = did_document.ok_or_else(|| AuthError::AccountNotFound)?;

    // Extract keys from DID document
    let did_keys = did_document.did_keys();
    if did_keys.is_empty() {
        return Err(AuthError::MissingKeys.into());
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
                            .map_err(|_| AuthError::InvalidClaims)?;
                        let header: Header = serde_json::from_slice(&header_bytes)
                            .map_err(|_| AuthError::InvalidClaims)?;
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
    #[error("Invalid JWT")]
    InvalidJWT,
    #[error("Invalid DID method: `{0}`")]
    InvalidDIDMethod(String),
    #[error("Invalid Claims")]
    InvalidClaims,
    #[error("Missing Issuer")]
    MissingIssuer,
    #[error("Missing keys in DID document")]
    MissingKeys,
    #[error("Authentication required")]
    AuthRequired,
    #[error("Account not found")]
    AccountNotFound,
    // #[error("Account has been taken down")]
    // AccountTakedown,
    // #[error("Account was deactivated")]
    // AccountDeactivated,
}

