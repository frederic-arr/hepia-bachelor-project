use std::str::FromStr as _;

use anyhow::{Result, anyhow};
use cos_proto_api::v1::{
    ConfigPullRequest,
    ConfigPushRequest,
    ConfigValidateRequest,
    ResourcesGetRequest,
    ResourcesListRequest,
    ResourcesReconcileNowRequest,
};
use cos_proto_api::{
    ConfigPullResponsePayload,
    ConfigValidateRequestPayload,
    ConfigValidateResponsePayload,
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

    pub async fn config_validate(
        &mut self,
        configs: Vec<SubResourceCreate<Value>>,
    ) -> Result<()> {
        let mut request = Request::new(ConfigValidateRequest {
            raw: serde_json::to_vec(&ConfigValidateRequestPayload {
                resources: configs,
            })?,
        });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        let raw = self.client.config_validate(request).await?.into_inner().raw;
        let response: ConfigValidateResponsePayload =
            serde_json::from_slice(&raw)?;

        match response {
            ConfigValidateResponsePayload::Ok => Ok(()),
            ConfigValidateResponsePayload::Error(e) => Err(anyhow!("{e}")),
        }
    }

    pub async fn config_push(
        &mut self,
        configs: &[SubResourceCreate<Value>],
    ) -> Result<()> {
        let mut request = Request::new(ConfigPushRequest {
            raw: serde_json::to_vec(&configs)?,
        });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        self.client.config_push(request).await?;
        Ok(())
    }

    pub async fn config_push_str(&mut self, s: &str) -> Result<()> {
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

        self.config_push(&configs).await
    }

    pub async fn config_pull(
        &mut self,
    ) -> Result<Vec<SubResourceCreate<Value>>> {
        let raw = self
            .client
            .config_pull(ConfigPullRequest { raw: vec![] })
            .await?
            .into_inner()
            .raw;
        let response: ConfigPullResponsePayload = serde_json::from_slice(&raw)?;
        Ok(response.resources)
    }

    pub async fn fs_write(&mut self) -> Result<()> {
        todo!()
    }

    pub async fn fs_list(&mut self) -> Result<()> {
        todo!()
    }

    pub async fn fs_read(&mut self) -> Result<()> {
        todo!()
    }

    pub async fn resources_list(&mut self) -> Result<Vec<Resource>> {
        let mut request = Request::new(ResourcesListRequest { raw: vec![] });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        let raw = self.client.resources_list(request).await?.into_inner().raw;
        let resources = serde_json::from_slice::<Vec<Resource>>(&raw)?;
        Ok(resources)
    }

    pub async fn resources_get(&mut self, key: &Key) -> Result<Resource> {
        let raw = serde_json::to_vec(key)?;

        let mut request = Request::new(ResourcesGetRequest { raw });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        let raw = self.client.resources_get(request).await?.into_inner().raw;
        let resource = serde_json::from_slice::<Resource>(&raw)?;

        Ok(resource)
    }

    pub async fn resources_reconcile_now(&mut self, key: &Key) -> Result<()> {
        let raw = serde_json::to_vec(key)?;

        let mut request = Request::new(ResourcesReconcileNowRequest { raw });
        if let Some(password) = &self.password {
            request.metadata_mut().append("x-auth", password.parse()?);
        }

        let _ = self.client.resources_reconcile_now(request).await?;
        Ok(())
    }

    pub async fn resources_force_delete(&mut self) -> Result<()> {
        todo!()
    }

    pub async fn system_reboot(&mut self) -> Result<()> {
        todo!()
    }
}
