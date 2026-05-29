use std::collections::HashSet;
use std::fmt::Write;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub schema: String,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec(pub Vec<u8>);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    UserConfig(UserConfig),
    DynamicResource(DynamicResource),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    Unset,
    Set(State),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfig {
    pub schema: String,
    pub name: String,
    pub spec: Spec,
    pub state: ResourceState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicResource {
    pub schema: String,
    pub name: String,
    pub owner: Identity,
    pub spec: Spec,
    pub state: ResourceState,
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.schema)?;
        f.write_char('/')?;
        f.write_str(&self.name)
    }
}

impl std::fmt::Debug for Spec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: rmpv::Value =
            rmp_serde::from_slice(&self.0).unwrap_or(rmpv::Value::Nil);
        f.debug_tuple("State").field(&v).finish()
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: rmpv::Value =
            rmp_serde::from_slice(&self.0).unwrap_or(rmpv::Value::Nil);
        f.debug_tuple("State").field(&v).finish()
    }
}
