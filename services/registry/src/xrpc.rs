use rocket::{http::{ContentType, Status}, response::Responder};
use std::{collections::HashMap, io::Cursor};
use did_method_plc::DIDPLC;
use serde_json::json;

pub struct XRPCServer<'p> {
    #[allow(dead_code)]
    plc: &'p DIDPLC,
    methods: HashMap<(XRPCMethodType, String), XRPCMethod>,
}

impl<'p> XRPCServer<'p> {
    pub fn new(plc: &'p DIDPLC) -> Self {
        Self {
            plc,
            methods: HashMap::new(),
        }
    }

    pub fn execute_method(&self, method_type: XRPCMethodType, method_name: String, request: &XRPCRequest) -> Result<String, XRPCError> {
        let method = self.methods.get(&(method_type, method_name));
        if method.is_none() {
            return Err(XRPCError::NotFound);
        }
        let method = method.unwrap();
        (method.handler)(request)
    }
}

pub struct XRPCRequest<'p> {
    pub plc: &'p DIDPLC,
    pub method: XRPCMethodType,
    pub params: HashMap<String, String>,
    pub body: Option<String>,
}

pub struct XRPCMethod {
    pub handler: fn(request: &XRPCRequest) -> Result<String, XRPCError>,
    pub requires_auth: bool,
}

#[derive(PartialEq, Eq, Hash)]
pub enum XRPCMethodType {
    GET,
    POST,
}

#[derive(thiserror::Error, Debug)]
pub enum XRPCError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not Found")]
    NotFound,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Bad Request")]
    BadRequest,
    #[error("Not Implemented")]
    NotImplemented,
    #[error("Payload Too Large")]
    PayloadTooLarge,
    #[error("Too Many Requests")]
    TooManyRequests,
    #[error("Bad Gateway")]
    BadGateway,
    #[error("Service Unavailable")]
    ServiceUnavailable,
    #[error("Gateway Timeout")]
    GatewayTimeout,

    #[error("{error}: {message}")]
    Custom {
        error: String,
        message: String,
    },
}

impl XRPCError {
    pub fn to_json(&self) -> String {
        match self {
            XRPCError::Unauthorized => json!({
                "error": "Unauthorized",
                "message": "The request requires authentication",
            }),
            XRPCError::Forbidden => json!({
                "error": "Forbidden",
                "message": "The request is forbidden",
            }),
            XRPCError::NotFound => json!({
                "error": "Not Found",
                "message": "The requested resource was not found",
            }),
            XRPCError::InternalServerError => json!({
                "error": "Internal Server Error",
                "message": "An internal server error occurred",
            }),
            XRPCError::BadRequest => json!({
                "error": "Bad Request",
                "message": "The request is malformed or invalid",
            }),
            XRPCError::NotImplemented => json!({
                "error": "Not Implemented",
                "message": "The requested method is not implemented",
            }),
            XRPCError::PayloadTooLarge => json!({
                "error": "Payload Too Large",
                "message": "The request payload is too large",
            }),
            XRPCError::TooManyRequests => json!({
                "error": "Too Many Requests",
                "message": "The request rate is too high",
            }),
            XRPCError::BadGateway => json!({
                "error": "Bad Gateway",
                "message": "The server encountered an unexpected condition",
            }),
            XRPCError::ServiceUnavailable => json!({
                "error": "Service Unavailable",
                "message": "The requested service is unavailable",
            }),
            XRPCError::GatewayTimeout => json!({
                "error": "Gateway Timeout",
                "message": "The gateway timed out",
            }),
            XRPCError::Custom { error, message } => json!({
                "error": error,
                "message": message,
            }),
        }.to_string()
    }

    fn status(&self) -> rocket::http::Status {
        match self {
            XRPCError::Unauthorized => Status::Unauthorized,
            XRPCError::Forbidden => Status::Forbidden,
            XRPCError::NotFound => Status::NotFound,
            XRPCError::InternalServerError => Status::InternalServerError,
            XRPCError::BadRequest => Status::BadRequest,
            XRPCError::NotImplemented => Status::NotImplemented,
            XRPCError::PayloadTooLarge => Status::PayloadTooLarge,
            XRPCError::TooManyRequests => Status::TooManyRequests,
            XRPCError::BadGateway => Status::BadGateway,
            XRPCError::ServiceUnavailable => Status::ServiceUnavailable,
            XRPCError::GatewayTimeout => Status::GatewayTimeout,
            XRPCError::Custom { .. } => Status::BadRequest,
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for XRPCError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        rocket::Response::build()
            .status(self.status())
            .header(ContentType::JSON)
            .sized_body(self.to_json().len(), Cursor::new(self.to_json()))
            .ok()
    }
}