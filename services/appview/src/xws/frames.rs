use rsky_common::struct_to_cbor;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Deserialize_repr, Serialize_repr)]
#[repr(i8)]
pub enum SocketFrameType {
    Error = -1,
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

pub enum SocketFrame {
    Data(SocketDataFrame<Value>),
    Error(SocketErrorFrame),
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct SocketDataFrame<T> {
    pub r#type: String,
    pub payload: T,
}

impl<T> SocketDataFrame<T> {
    pub fn new(r#type: String, payload: T) -> Self {
        Self {
            r#type,
            payload,
        }
    }
}

impl<T: serde::Serialize> SocketFrameSerializer for SocketDataFrame<T> {
    fn binary(&self) -> Result<Vec<u8>> {
        let mut value = struct_to_cbor(&SocketFrameHeader { op: SocketFrameType::Data, t: Some(self.r#type.clone()) })?;
        let mut payload = struct_to_cbor(&self.payload)?;
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
        Self {
            error,
            message,
        }
    }
}

impl SocketFrameSerializer for SocketErrorFrame {
    fn binary(&self) -> Result<Vec<u8>> {
        let mut value = struct_to_cbor(&SocketFrameHeader { op: SocketFrameType::Error, t: None })?;
        let mut payload = struct_to_cbor(&self)?;
        value.append(&mut payload);
        Ok(value)
    }
}