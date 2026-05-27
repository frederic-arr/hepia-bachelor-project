use cos_api_shared::Specification;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub type Identity = cos_api_shared::Identity;
pub type Spec = Vec<u8>;
pub type State = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub struct CreateDynamicResourceRequest<T>
where
    T: Specification,
{
    pub id: Identity,
    // pub owner: Identity,
    pub spec: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: DeserializeOwned"
))]
pub struct ReconcileDynamicResourceRequest<T>
where
    T: Specification,
{
    pub id: Identity,
    // pub owner: Identity,
    pub spec: T,
    pub state: T::State,
}
