use serde::{Deserialize, Serialize};

use crate::realtime::frames::SocketFrameType;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct SocketInFrame<T> {
    pub op: SocketFrameType,
    pub payload: T,
}

pub type SocketAuthFrame = SocketInFrame<Option<SocketAuthFramePayload>>;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(tag = "t")]
pub enum SocketInFramePayload {
    View(SocketViewFramePayload),
    ViewPermissions,
}

pub type SocketInAnyFrame = SocketInFrame<SocketInFramePayload>;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", crate = "rocket::serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct SocketAuthFramePayload {
    pub service_auth: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", crate = "rocket::serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct SocketViewFramePayload {
    pub campsite: String,
}