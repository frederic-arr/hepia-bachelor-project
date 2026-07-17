#![feature(decl_macro)]

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub schema: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Identity {
    Static(Key),
    Dynamic(Key),
    Shared(Key),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Running,
    Shutdown,
    Teardown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Unknown,
    Error(String),
    NotReady,
    Done,
    Ready,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource<T, U, V> {
    pub id: Identity,
    pub phase: Phase,
    pub status: Status,
    pub spec: T,
    pub derived_spec: U,
    pub state: V,
    pub children: Vec<Identity>,
    pub dependencies: Vec<Identity>,
    pub dependents: Vec<Identity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceResponse<V> {
    pub status: Status,
    pub state: V,
    pub children: Vec<Identity>,
    pub dependencies: Vec<Identity>,
}

pub macro assert_reconciliation_error($status:expr, $pat:expr) {
    ::std::assert_matches!($status, Status::Error(_));
    let Status::Error(err) = $status else {
        unreachable!()
    };

    assert!(
        err.contains($pat),
        "expected {:?} got {err:?}",
        $pat
    );
}
