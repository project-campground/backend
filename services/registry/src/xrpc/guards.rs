use crate::{config::AuthConfig, xrpc::auth::AuthToken};
use did_method_plc::{Keypair, DIDPLC};
use rocket::{
    data::{FromData, Outcome as DataOutcome, ToByteUnit},
    http::Status,
    request::{FromRequest, Outcome},
    Data, Request, State,
};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    Invalid,
    #[error("Expired token")]
    Expired,
    #[error("Missing")]
    Missing,
    #[error("Config is malformed")]
    MalformedConfig,
    #[error("Internal error")]
    Figment(#[from] rocket::figment::Error),
}

pub struct Authenticated(AuthToken);

impl Authenticated {
    pub fn token(&self) -> &AuthToken {
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
        let plc: &State<DIDPLC> = request.rocket().state().unwrap();
        let token_str = request.headers().get_one("Authorization");
        if token_str.is_none() {
            return Outcome::Error((Status::Unauthorized, AuthError::Missing));
        }
        let token_str = token_str.unwrap();
        let token_str = token_str.strip_prefix("Bearer ").unwrap_or(token_str);
        let key =
            Keypair::from_private_key(&config.secret_key).map_err(|_| AuthError::MalformedConfig);
        if key.is_err() {
            return Outcome::Error((Status::InternalServerError, AuthError::MalformedConfig));
        }
        let key = key.unwrap();

        let token = AuthToken::from_token(plc, &key, token_str)
            .await
            .map_err(|_| AuthError::Invalid);
        if token.is_err() {
            return Outcome::Error((Status::Unauthorized, AuthError::Invalid));
        }
        let token = token.unwrap();
        Outcome::Success(Authenticated(token))
    }
}

pub struct XRPCBody(String);

impl XRPCBody {
    pub fn body(&self) -> &str {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for XRPCBody {
    type Error = String;

    async fn from_data(_request: &'r Request<'_>, data: Data<'r>) -> DataOutcome<'r, Self> {
        let body = data.open(128.kilobytes());
        let body = body.into_string().await;
        if body.is_err() {
            return DataOutcome::Error((Status::BadRequest, "Invalid body".to_string()));
        }
        DataOutcome::Success(XRPCBody(body.unwrap().to_string()))
    }
}
