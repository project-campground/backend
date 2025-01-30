use rocket::response::Responder;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum XRPCError {
    #[error("(400) Bad request")]
    BadRequest,
    #[error("(401) Unauthorized")]
    Unauthorized,
    #[error("(403) Forbidden")]
    Forbidden,
    #[error("(404) Not found")]
    NotFound,
    #[error("(413) Payload too large")]
    PayloadTooLarge,
    #[error("(429) Too many requests")]
    TooManyRequests,
    #[error("(500) Internal server error")]
    InternalServerError,
    #[error("(501) Not implemented")]
    NotImplemented,
    #[error(transparent)]
    Other(anyhow::Error)
}

impl<'r> Responder<'r, 'static> for XRPCError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = match self {
            Self::BadRequest => rocket::http::Status::BadRequest,
            Self::Unauthorized => rocket::http::Status::Unauthorized,
            Self::Forbidden => rocket::http::Status::Forbidden,
            Self::NotFound => rocket::http::Status::NotFound,
            Self::PayloadTooLarge => rocket::http::Status::PayloadTooLarge,
            Self::TooManyRequests => rocket::http::Status::TooManyRequests,
            Self::InternalServerError => rocket::http::Status::InternalServerError,
            Self::NotImplemented => rocket::http::Status::NotImplemented,
            Self::Other(_) => rocket::http::Status::BadRequest,
        };
        
        let body = ErrorBody {
            error: status.reason().unwrap_or(&status.to_string()).to_string(),
            message: match self {
                Self::Other(err) => Some(err.to_string()),
                _ => None,
            },
        };
        let body = serde_json::to_string(&body).unwrap().into_bytes();
        
        rocket::Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(body.len(), std::io::Cursor::new(body.clone()))
            .ok()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

pub type Result<T, E = XRPCError> = std::result::Result<T, E>;