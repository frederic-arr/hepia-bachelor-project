mod conv;

use std::collections::HashSet;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

pub type Identity = cos_api_shared::Identity;
pub type Spec = Vec<u8>;
pub type State = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    UserConfig(UserConfigResource),
    Dynamic(DynamicResource),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceMeta {
    id: Identity,
    children: HashSet<Identity>,
    spec: ResourceSpec,
    state: ResourceState,
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
    Pending {
        state: State,
        state_at: SystemTime,
    },
    Ready {
        state: State,
        state_at: SystemTime,
    },
    Completed {
        state: State,
        state_at: SystemTime,
    },
    Error {
        error: String,
        state: State,
        state_at: SystemTime,
    },
    RefreshError {
        error: String,
        state: State,
        state_at: SystemTime,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfigResource {
    meta: ResourceMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicResource {
    meta: ResourceMeta,
    owner: Identity,
    dependencies: HashSet<Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfigResourceCreate {
    id: Identity,
    spec: Spec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicResourceCreate {
    id: Identity,
    owner: Identity,
    spec: Spec,
}
