use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubResourceCreate {
    pub schema: String,
    pub name: String,
    pub spec: Vec<u8>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Identity {
    pub schema: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileUserConfigRequest<Spec, State> {
    pub schema: String,
    pub name: String,
    pub spec: Spec,
    pub state: Option<State>,
    pub children: Vec<Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileDynamicResourceRequest<Spec, State> {
    pub schema: String,
    pub name: String,
    pub spec: Spec,
    pub state: Option<State>,
    pub children: Vec<Identity>,
    pub owner: Identity,
}
