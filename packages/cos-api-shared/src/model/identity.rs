use std::fmt::{Display, Write};

use serde::{Deserialize, Serialize};

use crate::proto::v1;

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct Identity {
    schema: String,
    name: String,
}

impl Identity {
    #[must_use]
    pub const fn new(schema: String, name: String) -> Self {
        Self { schema, name }
    }

    #[must_use]
    pub const fn schema(&self) -> &String {
        &self.schema
    }

    #[must_use]
    pub const fn name(&self) -> &String {
        &self.name
    }
}

impl Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.schema)?;
        f.write_char('/')?;
        f.write_str(&self.name)
    }
}

impl From<Identity> for v1::Identity {
    fn from(value: Identity) -> Self {
        Self {
            name: value.name,
            schema: value.schema,
        }
    }
}

impl TryFrom<v1::Identity> for Identity {
    type Error = String;

    fn try_from(value: v1::Identity) -> Result<Self, Self::Error> {
        let id = Self {
            name: value.name,
            schema: value.schema,
        };

        Ok(id)
    }
}

impl TryFrom<Option<v1::Identity>> for Identity {
    type Error = String;

    fn try_from(value: Option<v1::Identity>) -> Result<Self, Self::Error> {
        value.ok_or_else(|| "Identity is required".to_string())?.try_into()
    }
}
