use rocket::{http::{ContentType, Status}, response::Responder};
use std::io::Cursor;
use serde_json::json;

pub mod guards;
pub mod auth;

#[derive(thiserror::Error, Debug)]
pub enum XRPCError {
    #[error("Unauthorized: {message}")]
    Unauthorized {message: String},
    #[error("Forbidden: {message}")]
    Forbidden {message: String},
    #[error("Not Found: {message}")]
    NotFound {message: String},
    #[error("Internal Server Error: {message}")]
    InternalServerError {message: String},
    #[error("Bad Request: {message}")]
    BadRequest {message: String},
    #[error("Not Implemented: {message}")]
    NotImplemented {message: String},
    #[error("Payload Too Large: {message}")]
    PayloadTooLarge {message: String},
    #[error("Too Many Requests: {message}")]
    TooManyRequests {message: String},
    #[error("Bad Gateway: {message}")]
    BadGateway {message: String},
    #[error("Service Unavailable: {message}")]
    ServiceUnavailable {message: String},
    #[error("Gateway Timeout: {message}")]
    GatewayTimeout {message: String},

    #[error("{error}: {message}")]
    Custom {
        error: String,
        message: String,
    },
}

impl XRPCError {
    pub fn to_json(&self) -> String {
        match self {
            XRPCError::Unauthorized { message } => json!({
                "error": "Unauthorized",
                "message": message,
            }),
            XRPCError::Forbidden { message } => json!({
                "error": "Forbidden",
                "message": message,
            }),
            XRPCError::NotFound { message } => json!({
                "error": "Not Found",
                "message": message,
            }),
            XRPCError::InternalServerError { message } => json!({
                "error": "Internal Server Error",
                "message": message,
            }),
            XRPCError::BadRequest { message } => json!({
                "error": "Bad Request",
                "message": message,
            }),
            XRPCError::NotImplemented { message } => json!({
                "error": "Not Implemented",
                "message": message,
            }),
            XRPCError::PayloadTooLarge { message } => json!({
                "error": "Payload Too Large",
                "message": message,
            }),
            XRPCError::TooManyRequests { message } => json!({
                "error": "Too Many Requests",
                "message": message,
            }),
            XRPCError::BadGateway { message } => json!({
                "error": "Bad Gateway",
                "message": message,
            }),
            XRPCError::ServiceUnavailable { message } => json!({
                "error": "Service Unavailable",
                "message": message,
            }),
            XRPCError::GatewayTimeout { message } => json!({
                "error": "Gateway Timeout",
                "message": message,
            }),
            XRPCError::Custom { error, message } => json!({
                "error": error,
                "message": message,
            }),
        }.to_string()
    }

    fn status(&self) -> rocket::http::Status {
        match self {
            XRPCError::Unauthorized { .. } => Status::Unauthorized,
            XRPCError::Forbidden { .. } => Status::Forbidden,
            XRPCError::NotFound { .. } => Status::NotFound,
            XRPCError::InternalServerError { .. } => Status::InternalServerError,
            XRPCError::BadRequest { .. } => Status::BadRequest,
            XRPCError::NotImplemented { .. } => Status::NotImplemented,
            XRPCError::PayloadTooLarge { .. } => Status::PayloadTooLarge,
            XRPCError::TooManyRequests { .. } => Status::TooManyRequests,
            XRPCError::BadGateway { .. } => Status::BadGateway,
            XRPCError::ServiceUnavailable { .. } => Status::ServiceUnavailable,
            XRPCError::GatewayTimeout { .. } => Status::GatewayTimeout,
            XRPCError::Custom { .. } => Status::BadRequest,
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for XRPCError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let json = self.to_json();
        rocket::Response::build()
            .status(self.status())
            .header(ContentType::JSON)
            .sized_body(json.len(), Cursor::new(json))
            .ok()
    }
}