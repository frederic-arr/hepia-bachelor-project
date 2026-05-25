use cos_api_shared::{Identity, Specification, State};
use cos_api_sysmgr::proto::v1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Payload {
    bytes: Vec<u8>,
}

impl Serialize for Payload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        rmp_serde::from_slice::<rmpv::Value>(&self.bytes)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        rmp_serde::to_vec(&rmpv::Value::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
            .map(|bytes| Self { bytes })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateConfig {
    #[serde(flatten)]
    pub id: Identity,

    #[serde(flatten)]
    pub spec: Payload,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateResource {
    pub id: Identity,
    pub owner: Identity,
    pub spec: Vec<u8>,
}

impl From<Vec<u8>> for Payload {
    fn from(value: Vec<u8>) -> Self {
        Self { bytes: value }
    }
}

impl Specification for Payload {
    type State = Payload;

    fn into_bytes(self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        Ok(self.bytes)
    }
}

impl State for Payload {
    fn into_bytes(self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        Ok(self.bytes)
    }
}

impl TryFrom<v1::ResourceCreateDynamicRequest> for CreateResource {
    type Error = String;

    fn try_from(
        value: v1::ResourceCreateDynamicRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into()?,
            owner: value.owner.try_into()?,
            spec: value.spec,
        })
    }
}
