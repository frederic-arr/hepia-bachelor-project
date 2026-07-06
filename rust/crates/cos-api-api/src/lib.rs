use serde::{Deserialize, Serialize};

pub mod proto;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiResource {
    pub schema: String,
    pub name: String,
    pub spec: Vec<u8>,
}
