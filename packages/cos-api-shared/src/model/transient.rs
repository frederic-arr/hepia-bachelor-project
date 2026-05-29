use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Specification:
    Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Send + Sync
{
    type State: State;

    const SCHEMA: &str;

    fn into_bytes(self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&self)
    }
}

pub trait State:
    Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Send + Sync
{
    fn into_bytes(self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&self)
    }
}

impl Specification for rmpv::Value {
    type State = Self;

    const SCHEMA: &str = ".containeros.internal.raw";
}

impl State for rmpv::Value {}
