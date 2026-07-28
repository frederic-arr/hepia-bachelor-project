use std::str::FromStr as _;

use anyhow::Result;
use cos_proto_api::v1::{
    GetResourceRequest,
    ListResourcesRequest,
    PushConfigRequest,
    ReconcileNowRequest,
};
use cos_proto_api_client::v1::ApiServiceClient;
use cos_proto_reconciler::{Identity, PrivateIdentity};
pub use cos_proto_reconciler::{Key, SubResourceCreate, TerminalResource};
use serde::{Deserialize, Serialize};
pub use serde_json::Value;
use tonic::Request;
use tonic::transport::{Channel, Endpoint};

pub struct CosClient {
    password: Option<String>,
    client: ApiServiceClient<Channel>,
}

pub type Resource = TerminalResource<Value, Value, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigResource {
    pub schema: String,
    pub name: Option<String>,

    #[serde(flatten)]
    pub spec: Value,
}

impl CosClient {
    pub fn new(addr: &str, password: Option<String>) -> Result<Self> {
        let conn = Endpoint::from_str(addr)?.connect_lazy();
        let client = ApiServiceClient::new(conn);

        Ok(Self { password, client })
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.password = password;
    }

    pub async fn reconcile(&mut self, key: &Key) -> Result<()> {
        let raw = serde_json::to_vec(key)?;

        let mut request = Request::new(ReconcileNowRequest { raw });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        let _ = self.client.reconcile_now(request).await?;
        Ok(())
    }

    pub async fn push_str(&mut self, s: &str) -> Result<()> {
        let configs = serde_yaml::Deserializer::from_str(s)
            .map(ConfigResource::deserialize)
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(|v| SubResourceCreate {
                id: Identity::Private(PrivateIdentity::Static(Key {
                    schema: v.schema,
                    name: v.name,
                })),
                spec: v.spec,
            })
            .collect::<Vec<_>>();

        self.push(&configs).await
    }

    pub async fn push(
        &mut self,
        configs: &[SubResourceCreate<Value>],
    ) -> Result<()> {
        let mut request = Request::new(PushConfigRequest {
            raw: serde_json::to_vec(&configs)?,
        });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        self.client.push_config(request).await?;
        Ok(())
    }

    pub async fn list(&mut self) -> Result<Vec<Resource>> {
        let mut request = Request::new(ListResourcesRequest { raw: vec![] });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        let raw = self.client.list_resources(request).await?.into_inner().raw;
        let resources = serde_json::from_slice::<Vec<Resource>>(&raw)?;
        Ok(resources)
    }

    pub async fn get_resource(&mut self, key: &Key) -> Result<Resource> {
        let raw = serde_json::to_vec(key)?;

        let mut request = Request::new(GetResourceRequest { raw });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        let raw = self.client.get_resource(request).await?.into_inner().raw;
        let resource = serde_json::from_slice::<Resource>(&raw)?;

        Ok(resource)
    }
}
