// mod conv;

use std::collections::HashSet;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

pub type Identity = cos_api_shared::Identity;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec(pub Vec<u8>);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    UserConfig(UserConfigResource),
    Dynamic(DynamicResource),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceMeta {
    pub id: Identity,
    // pub children: HashSet<Identity>,
    pub spec: ResourceSpec,
    pub state: ResourceState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceSpec {
    Running { spec: Spec },
    Draining { spec: Spec },
    Deleting { spec: Spec },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    Unset,
    // Pending {
    //     state: State,
    //     state_at: SystemTime,
    // },
    Ready {
        state: State,
        // state_at: SystemTime,
    },
    // Completed {
    //     state: State,
    //     state_at: SystemTime,
    // },
    Error {
        error: String,
        state: State,
        // state_at: SystemTime,
    },
    // RefreshError {
    //     error: String,
    //     state: State,
    //     state_at: SystemTime,
    // },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfigResource {
    pub meta: ResourceMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicResource {
    pub meta: ResourceMeta,
    // pub owner: Identity,
    // pub dependencies: HashSet<Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfigResourceCreate {
    pub id: Identity,
    pub spec: Spec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicResourceCreate {
    pub id: Identity,
    // pub owner: Identity,
    pub spec: Spec,
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
