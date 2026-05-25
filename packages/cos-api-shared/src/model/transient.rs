use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub trait Specification:
    Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq
{
    type State: State;

    fn into_bytes(self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&self)
    }
}

pub trait State:
    Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq
{
    fn into_bytes(self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&self)
    }
}

impl Specification for rmpv::Value {
    type State = rmpv::Value;
}

impl State for rmpv::Value {}
