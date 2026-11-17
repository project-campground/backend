use anyhow::Result;
use rsky_common::struct_to_cbor;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, PartialEq, Deserialize_repr, Serialize_repr)]
#[repr(i8)]
pub enum SocketFrameType {
    Error = -1,
    Auth = 0,
    Data = 1,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct SocketFrameHeader {
    pub op: SocketFrameType,
    pub t: Option<String>,
}

pub trait SocketFrameSerializer {
    fn binary(&self) -> Result<Vec<u8>>;
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct SocketDataFrame<'a, T> {
    pub r#type: String,
    pub payload: &'a T,
}

impl<'a, T> SocketDataFrame<'a, T> {
    pub fn new(r#type: String, payload: &'a T) -> Self {
        Self { r#type, payload }
    }
}

impl<'a, T: serde::Serialize> SocketFrameSerializer for SocketDataFrame<'a, T> {
    fn binary(&self) -> Result<Vec<u8>> {
        let mut value = struct_to_cbor(&SocketFrameHeader {
            op: SocketFrameType::Data,
            t: Some(self.r#type.clone()),
        })?;
        let mut payload = struct_to_cbor(self.payload)?;
        value.append(&mut payload);
        Ok(value)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct SocketErrorFrame {
    pub error: String,
    pub message: Option<String>,
}

impl SocketErrorFrame {
    pub fn new(error: String, message: Option<String>) -> Self {
        Self { error, message }
    }
    pub fn from_error_message(error: &str, message: &str) -> (Result<Vec<u8>>, ws::Message) {
        (
            SocketErrorFrame::new(error.to_string(), Some(message.to_string())).binary(),
            ws::Message::Close(Some(ws::frame::CloseFrame {
                code: ws::frame::CloseCode::Error,
                reason: std::borrow::Cow::Owned(message.to_string()),
            })),
        )
    }
}

impl SocketFrameSerializer for SocketErrorFrame {
    fn binary(&self) -> Result<Vec<u8>> {
        let mut value = struct_to_cbor(&SocketFrameHeader {
            op: SocketFrameType::Error,
            t: None,
        })?;
        let mut payload = struct_to_cbor(&self)?;
        value.append(&mut payload);
        Ok(value)
    }
}
