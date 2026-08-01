#![feature(decl_macro)]

use std::collections::HashSet;
use std::fmt::Write as _;
use std::hash::Hash;
use std::str::FromStr;

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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Key {
    pub schema: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Identity {
    Private(PrivateIdentity),
    Shared(Key),
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub enum PrivateIdentity {
    Ephemeral(Key),
    Dynamic(Key),
    Static(Key),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Running,
    Shutdown,
    PendingDeletion,
    Deleting,
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

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
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
            Self::Private(key) => key.key(),
            Self::Shared(key) => key,
        }
    }

    #[must_use]
    pub fn schema(&self) -> &String {
        &self.key().schema
    }
}

impl PrivateIdentity {
    #[must_use]
    pub fn key(&self) -> &Key {
        match self {
            Self::Static(key) | Self::Dynamic(key) | Self::Ephemeral(key) => {
                key
            }
        }
    }
}

impl PartialEq for PrivateIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Hash for PrivateIdentity {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.key().hash(state);
    }
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = match self {
            Self::Private(PrivateIdentity::Dynamic(key)) => {
                f.write_str("dyn#")?;
                key
            }
            Self::Private(PrivateIdentity::Ephemeral(key)) => {
                f.write_str("tmp#")?;
                key
            }
            Self::Private(PrivateIdentity::Static(key)) => {
                f.write_str("cfg#")?;
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

impl FromStr for Key {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = match s.split_once('/') {
            Some((schema, name)) => Self {
                schema: schema.to_owned(),
                name: Some(name.to_owned()),
            },
            None => Self {
                schema: s.to_owned(),
                name: None,
            },
        };

        Ok(key)
    }
}
