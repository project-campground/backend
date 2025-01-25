/**
 * Implementation from https://github.com/blacksky-algorithms/rsky
 * License: https://github.com/blacksky-algorithms/rsky/blob/main/LICENSE
 */

use crate::xrpc_server::auth::{verify_jwt as verify_service_jwt_server, ServiceJwtPayload};
use rsky_identity::did::atproto_data::{get_did_key_from_multibase, VerificationMaterial};
use rocket::request::{FromRequest, Outcome, Request};
use rsky_identity::types::DidDocument;
use jwt_simple::claims::Audiences;
use crate::SharedIdResolver;
use jwt_simple::prelude::*;
use anyhow::{bail, Result};
use rocket::http::Status;
use thiserror::Error;
use rocket::State;

use crate::config::CORE_CONFIG;

const BEARER: &str = "Bearer ";

#[derive(PartialEq, Clone, Debug)]
pub enum AuthScope {
    Access,
    Refresh,
    AppPass,
    AppPassPrivileged,
    SignupQueued,
}

#[derive(Clone)]
pub struct Credentials {
    pub r#type: String,
    pub did: Option<String>,
    pub scope: Option<AuthScope>,
    pub audience: Option<String>,
    pub token_id: Option<String>,
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub is_privileged: Option<bool>,
}

#[derive(Clone)]
pub struct AccessOutput {
    pub credentials: Option<Credentials>,
    pub artifacts: Option<String>,
}

pub struct ValidatedBearer {
    pub did: String,
    pub scope: AuthScope,
    pub token: String,
    pub payload: JwtPayload,
    pub audience: Option<String>,
}

#[derive(Clone)]
pub struct JwtPayload {
    pub scope: AuthScope,
    pub sub: Option<String>,
    pub aud: Option<Audiences>,
    pub exp: Option<Duration>,
    pub iat: Option<Duration>,
    pub jti: Option<String>,
}

pub struct ServiceJwtOpts {
    pub aud: Option<String>,
    pub iss: Option<Vec<String>>,
}

pub struct VerifiedServiceJwt {
    pub aud: String,
    pub iss: String,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("BadJwt: `{0}`")]
    BadJwt(String),
    #[error("BadJwtAudience: `{0}`")]
    BadJwtAudience(String),
    #[error("UntrustedIss: `{0}`")]
    UntrustedIss(String),
    #[error("AuthRequired: `{0}`")]
    AuthRequired(String),
    #[error("AccountNotFound: `{0}`")]
    AccountNotFound(String),
    #[error("AccountTakedown: `{0}`")]
    AccountTakedown(String),
    #[error("AccountDeactivated: `{0}`")]
    AccountDeactivated(String),
}

pub struct UserDidAuth {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserDidAuth {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let id_resolver = req.guard::<&State<SharedIdResolver>>().await.unwrap();
        match verify_service_jwt(
            req,
            id_resolver,
            ServiceJwtOpts {
                aud: Some(CORE_CONFIG.did()),
                iss: None,
            },
        )
        .await
        {
            Ok(payload) => Outcome::Success(UserDidAuth {
                access: AccessOutput {
                    credentials: Some(Credentials {
                        r#type: "user_did".to_string(),
                        did: None,
                        scope: None,
                        audience: None,
                        token_id: None,
                        aud: Some(payload.aud),
                        iss: Some(payload.iss),
                        is_privileged: None,
                    }),
                    artifacts: None,
                },
            }),
            Err(error) => {
                Outcome::Error((Status::BadRequest, AuthError::BadJwt(error.to_string())))
            }
        }
    }
}

pub struct UserDidAuthOptional {
    pub access: Option<AccessOutput>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserDidAuthOptional {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if is_bearer_token(req) {
            match UserDidAuth::from_request(req).await {
                Outcome::Success(output) => Outcome::Success(UserDidAuthOptional {
                    access: Some(output.access),
                }),
                Outcome::Error(err) => Outcome::Error(err),
                _ => panic!("Unexpected outcome during UserDidAuthOptional"),
            }
        } else {
            Outcome::Success(UserDidAuthOptional { access: None })
        }
    }
}

pub fn get_did(doc: &DidDocument) -> String {
    doc.id.clone()
}

pub fn get_verification_material(
    doc: &DidDocument,
    key_id: &String,
) -> Option<VerificationMaterial> {
    let did = get_did(doc);
    let keys = &doc.verification_method;
    if let Some(keys) = keys {
        let found = keys
            .into_iter()
            .find(|key| key.id == format!("#{key_id}") || key.id == format!("{did}#{key_id}"));
        match found {
            Some(found) if found.public_key_multibase.is_some() => {
                let found = found.clone();
                Some(VerificationMaterial {
                    r#type: found.r#type,
                    public_key_multibase: found.public_key_multibase.unwrap(),
                })
            }
            _ => None,
        }
    } else {
        None
    }
}

pub async fn verify_service_jwt<'r>(
    request: &'r Request<'_>,
    id_resolver: &State<SharedIdResolver>,
    opts: ServiceJwtOpts,
) -> Result<VerifiedServiceJwt> {
    let get_signing_key = |iss: String, force_refresh: bool| -> Result<String> {
        match &opts.iss {
            Some(opts_iss) if opts_iss.contains(&iss) => bail!("UntrustedIss: Untrusted issuer"),
            _ => (),
        }
        let parts = iss.split("#").collect::<Vec<&str>>();
        if let (Some(did), Some(service_id)) = (parts.get(0), parts.get(1)) {
            let (did, service_id) = (did.to_string(), *service_id);
            let key_id = if service_id == "atproto_labeler" {
                "atproto_label"
            } else {
                "atproto"
            };
            let mut lock = futures::executor::block_on(id_resolver.id_resolver.write());
            let did_doc: Result<DidDocument> =
                futures::executor::block_on(lock.did.ensure_resolve(&did, Some(force_refresh)));
            let did_doc: DidDocument = match did_doc {
                Err(err) => bail!("could not resolve iss did: `{err}`"),
                Ok(res) => res,
            };
            match get_verification_material(&did_doc, &key_id.to_string()) {
                None => bail!("missing or bad key in did doc"),
                Some(parsed_key) => match get_did_key_from_multibase(parsed_key)? {
                    None => bail!("missing or bad key in did doc"),
                    Some(did_key) => Ok(did_key),
                },
            }
        } else {
            bail!("could not resolve iss did")
        }
    };

    match bearer_token_from_req(request)? {
        None => bail!("MissingJwt: missing jwt"),
        Some(jwt_str) => {
            let payload: ServiceJwtPayload =
                verify_service_jwt_server(jwt_str, opts.aud, get_signing_key).await?;
            Ok(VerifiedServiceJwt {
                iss: payload.iss,
                aud: payload.aud,
            })
        }
    }
}

pub fn bearer_token_from_req(request: &Request) -> Result<Option<String>> {
    match request.headers().get_one("authorization") {
        Some(header) if !header.starts_with("Bearer ") => Ok(None),
        Some(header) => {
            let slice = &header["Bearer ".len()..];
            Ok(Some(slice.to_string()))
        }
        None => Ok(None),
    }
}

pub fn is_bearer_token(request: &Request) -> bool {
    match request.headers().get_one("Authorization") {
        None => false,
        Some(auth_header) => auth_header.starts_with(BEARER),
    }
}