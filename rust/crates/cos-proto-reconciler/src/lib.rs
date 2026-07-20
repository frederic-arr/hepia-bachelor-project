#![feature(decl_macro)]

use std::collections::HashSet;
use std::fmt::Write as _;

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

    tonic::include_proto!("containeros.reconciler.v1");
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub schema: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Identity {
    Dynamic(Key),
    Shared(Key),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Running,
    Shutdown,
    Teardown,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Unknown,
    Error(StatusError),
    NotReady,
    Done,
    Ready,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusError {
    /// No client exist for this resource.
    NoClient,

    /// The reconciliation took too long and the state manager aborted.
    TimedOut,

    /// There was an error in the state manager.
    Internal,

    /// The response returned by the reconciler was invalid.
    Invalid,

    /// The reconciler returned an error at the gRPC level indicating that the
    /// reconciliation did not happen as expected.
    Transport(String),

    /// The reconciler returned an error that is "expected".
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource<T, U, V> {
    pub id: Identity,
    pub phase: Phase,
    pub status: Status,
    pub spec: T,
    pub derived_spec: U,
    pub state: Option<V>,
    pub children: Vec<TerminalResource<Value, Value, Value>>,
    pub dependencies: Vec<TerminalResource<Value, Value, Value>>,
    pub dependents: Vec<TerminalResource<Value, Value, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalResource<T, U, V> {
    pub id: Identity,
    pub phase: Phase,
    pub status: Status,
    pub spec: T,
    pub derived_spec: U,
    pub state: Option<V>,
    pub children: HashSet<Identity>,
    pub dependencies: HashSet<Identity>,
    pub dependents: HashSet<Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceResponse<V> {
    pub status: Status,
    pub state: Option<V>,
    pub children: Vec<SubResourceCreate<Value>>,
    pub dependencies: HashSet<Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubResourceCreate<T> {
    pub id: Identity,
    pub spec: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateResponse<U> {
    pub derived_spec: U,
    pub children: Vec<SubResourceCreate<Value>>,
    pub dependencies: HashSet<Identity>,
}

pub macro assert_reconciliation_error($status:expr, $pat:expr) {
    ::std::assert_matches!($status, Status::Error(_));
    let Status::Error(err) = $status else {
        unreachable!()
    };

    // TODO:
    // assert!(
    //     err.contains($pat),
    //     "expected {:?} got {err:?}",
    //     $pat
    // );
}

impl Identity {
    #[must_use]
    pub fn key(&self) -> &Key {
        match self {
            Self::Dynamic(key) | Self::Shared(key) => key,
        }
    }

    #[must_use]
    pub fn schema(&self) -> &String {
        &self.key().schema
    }
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = match self {
            Self::Dynamic(key) => {
                f.write_str("dyn#")?;
                key
            }
            Self::Shared(key) => {
                f.write_str("sh#")?;
                key
            }
        };

        key.fmt(f)
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.schema)?;
        if let Some(name) = &self.name {
            f.write_char('/')?;
            f.write_str(name)?;
        }

        Ok(())
    }
}

impl<T> From<T> for StatusError
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self::Other(value.to_string())
    }
}
