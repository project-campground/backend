use rocket::{
    http::Status, request::{FromRequest, Outcome}
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::{
    config::AuthConfig,
    xrpc::auth::{validate_token, TokenType}
};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    Invalid,
    #[error("Expired token")]
    Expired,
    #[error("Missing")]
    Missing,
    #[error("Internal error")]
    Figment(#[from] rocket::figment::Error)
}

pub struct Authenticated(String);

impl Authenticated {
    pub fn token(&self) -> &str {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticated {
    type Error = AuthError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let config: AuthConfig = match request.rocket().figment().extract_inner("auth") {
            Ok(c) => c,
            Err(e) => return Outcome::Error((Status::InternalServerError, AuthError::Figment(e))),
        };
        let token_str = request.headers().get_one("Authorization");
        if token_str.is_none() {
            return Outcome::Error((Status::Unauthorized, AuthError::Missing));
        }
        let token_str = token_str.unwrap();
        let token_str = token_str.strip_prefix("Bearer ").unwrap_or(token_str);
        let key: Hmac<Sha256> = Hmac::new_from_slice(&config.secret_key).unwrap();

        let token = match validate_token(key, TokenType::Access, token_str.to_string()) {
            Ok(did) => did,
            Err(_) => return Outcome::Error((Status::Unauthorized, AuthError::Invalid)),
        };
        Outcome::Success(Authenticated(token))
    }
}

pub struct ContentType(String);

impl ContentType {
    pub fn content_type(&self) -> &str {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentType {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let content_type = request.content_type();
        if content_type.is_none() {
            return Outcome::Error((Status::UnsupportedMediaType, ()));
        }
        let content_type = content_type.unwrap();
        Outcome::Success(ContentType(content_type.to_string()))
    }
}