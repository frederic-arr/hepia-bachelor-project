use std::path::PathBuf;

use cos_proto_reconciler::{SubResourceCreate, TerminalResource};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod v1 {
    #![allow(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::restriction,
        warnings,
        unknown_lints
    )]

    tonic::include_proto!("containeros.api.v1");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidateRequestPayload {
    pub resources: Vec<SubResourceCreate<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValidateResponsePayload {
    Ok,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPushRequestPayload {
    pub resources: Vec<SubResourceCreate<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPushResponsePayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPullRequestPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPullResponsePayload {
    pub resources: Vec<SubResourceCreate<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsWriteRequestPayload {
    pub volume: String,
    pub path: PathBuf,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsWriteResponsePayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsListRequestPayload {
    pub volume: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsListResponsePayload {
    pub entries: Vec<(bool, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsReadRequestPayload {
    pub volume: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsReadResponsePayload {
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListRequestPayload {
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResponsePayload {
    pub resources: Vec<TerminalResource<Value, Value, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesGetRequestPayload {
    pub schema: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesGetResponsePayload {
    pub resource: TerminalResource<Value, Value, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReconcileNowRequestPayload {
    pub schema: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReconcileNowResponsePayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesForceDeleteRequestPayload {
    pub schema: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesForceDeleteResponsePayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRebootRequestPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRebootResponsePayload;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigResource {
    pub schema: String,
    pub name: Option<String>,

    #[serde(flatten)]
    pub spec: Value,
}
