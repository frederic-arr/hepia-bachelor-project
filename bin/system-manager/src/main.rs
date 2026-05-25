#![feature(map_try_insert)]

mod model;

use std::collections::HashMap;

use cos_api_shared::*;
use cos_api_sysmgr::proto::v1;
use cos_api_sysmgr_server::proto::v1 as v1_svc;
use invariant_macros::invariant;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub use crate::model::*;

#[derive(Default)]
pub struct SystemManagerInner {
    resources: HashMap<Identity, Resource<rmpv::Value>>,
}

#[derive(Default)]
pub struct SystemManagerService(RwLock<SystemManagerInner>);

impl SystemManagerInner {
    fn new() -> Self {
        Self::default()
    }

    fn config_create(&mut self, req: CreateConfig) -> Result<(), String> {
        let id = req.id.clone();
        let meta = ResourceMeta::<rmpv::Value>::new(
            req.id,
            req.spec.try_into().unwrap(),
        );
        let resource = UserConfigResource::new(meta);

        self.resources
            .try_insert(id, resource.try_into().unwrap())
            .map(|_| ())
            .map_err(|_| "cannot create a duplicate resource".to_string())
    }

    fn resource_create(&mut self, req: CreateResource) -> Result<(), String> {
        let id = req.id.clone();
        let meta = ResourceMeta::<rmpv::Value>::new(
            req.id,
            req.spec.try_into().unwrap(),
        );
        let resource = DynamicResource::new(meta, req.owner);

        if self.resources.contains_key(resource.meta().id()) {
            return Err("cannot create a duplicate resource".to_string());
        }

        let inserted_in_owner = self
            .resources
            .get_mut(resource.owner())
            .ok_or_else(|| {
                "the resource owner should be a valid reference".to_string()
            })?
            .meta_mut()
            .children_mut()
            .insert(resource.meta().id().clone());

        invariant!(
            inserted_in_owner,
            "owner {} contains child {id} that does not exist in the resource \
             store",
            resource.owner(),
        );

        let exists_in_store = self
            .resources
            .insert(
                resource.meta().id().clone(),
                resource.try_into().unwrap(),
            )
            .is_some();

        invariant!(
            !exists_in_store,
            "resource {id} was absent during contains_key check but present \
             on insert"
        );

        Ok(())
    }

    fn resource_read(&self, id: &Identity) -> Option<&Resource<rmpv::Value>> {
        self.resources.get(id)
    }

    fn resource_read_user_config(
        &self,
        id: &Identity,
    ) -> Option<&UserConfigResource<rmpv::Value>> {
        self.resource_read(id)?.maybe_user_config()
    }

    fn resource_read_dynamic(
        &self,
        id: &Identity,
    ) -> Option<&DynamicResource<rmpv::Value>> {
        self.resource_read(id)?.maybe_dynamic()
    }
}

impl SystemManagerService {
    async fn read(&self) -> RwLockReadGuard<'_, SystemManagerInner> {
        self.0.read().await
    }

    async fn write(&self) -> RwLockWriteGuard<'_, SystemManagerInner> {
        self.0.write().await
    }
}

#[tonic::async_trait]
impl v1_svc::SystemManagerService for SystemManagerService {
    async fn resource_create_dynamic(
        &self,
        request: Request<v1::ResourceCreateDynamicRequest>,
    ) -> Result<Response<v1::ResourceCreateDynamicResponse>, Status> {
        let resource: CreateResource = request.into_inner().try_into().unwrap();

        let mut inner = self.write().await;
        inner.resource_create(resource).map_err(Status::internal)?;
        drop(inner);

        Ok(Response::new(
            v1::ResourceCreateDynamicResponse {},
        ))
    }

    async fn resource_read(
        &self,
        request: Request<v1::ResourceReadRequest>,
    ) -> Result<Response<v1::ResourceReadResponse>, Status> {
        let id: Identity = request.into_inner().id.try_into().unwrap();

        let inner = self.read().await;
        let resource = inner.resource_read(&id).cloned();
        drop(inner);

        resource
            .ok_or_else(|| {
                Status::not_found(format!("resource {id} was not found"))
            })
            .map(|res| v1::ResourceReadResponse {
                resource: Some(res.try_into().unwrap()),
            })
            .map(Response::new)
    }

    async fn resource_update_dynamic_spec(
        &self,
        request: Request<v1::ResourceUpdateDynamicSpecRequest>,
    ) -> Result<Response<v1::ResourceUpdateDynamicSpecResponse>, Status> {
        todo!()
    }

    async fn resource_update_status(
        &self,
        request: Request<v1::ResourceUpdateStatusRequest>,
    ) -> Result<Response<v1::ResourceUpdateStatusResponse>, Status> {
        todo!()
    }

    async fn resource_delete_dynamic(
        &self,
        request: Request<v1::ResourceDeleteDynamicRequest>,
    ) -> Result<Response<v1::ResourceDeleteDynamicResponse>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let system_manager = SystemManagerService::default();

    println!("GreeterServer listening on {addr}");

    Server::builder()
        .add_service(v1_svc::SystemManagerServiceServer::new(
            system_manager,
        ))
        .serve(addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    mod config {
        use super::*;

        mod create {
            use super::*;

            fn setup() -> (SystemManagerInner, CreateConfig) {
                (SystemManagerInner::new(), CreateConfig::default())
            }

            #[test]
            fn basic_succeeds() {
                let (mut svc, res) = setup();

                svc.config_create(res.clone()).unwrap();

                assert_eq!(svc.resources.len(), 1);
                assert_matches!(
                    svc.resource_read_user_config(&res.id),
                    Some(_)
                );
            }

            #[test]
            fn fails_if_already_exists() {
                let (mut svc, res) = setup();
                svc.config_create(res.clone()).unwrap();

                svc.config_create(res.clone()).unwrap_err();

                assert_eq!(svc.resources.len(), 1);
                assert_matches!(
                    svc.resource_read_user_config(&res.id),
                    Some(_)
                );
            }
        }
    }

    mod dynamic {
        use super::*;

        mod create {
            use super::*;

            fn setup() -> (SystemManagerInner, CreateResource) {
                let mut svc = SystemManagerInner::new();
                let cfg = CreateConfig {
                    id: Identity::new(
                        "my-schema".to_string(),
                        "my-id".to_string(),
                    ),
                    spec: vec![],
                };
                svc.config_create(cfg.clone()).unwrap();
                (
                    svc,
                    CreateResource {
                        id: Identity::default(),
                        owner: cfg.id,
                        spec: vec![],
                    },
                )
            }

            #[test]
            fn basic_succeeds() {
                let (mut svc, res) = setup();

                svc.resource_create(res.clone()).unwrap();

                assert_eq!(svc.resources.len(), 2);
                assert_matches!(svc.resource_read_dynamic(&res.id), Some(_));
            }

            #[test]
            fn fails_if_already_exists() {
                let (mut svc, res) = setup();
                svc.resource_create(res.clone()).unwrap();

                svc.resource_create(res.clone()).unwrap_err();

                assert_eq!(svc.resources.len(), 2);
                assert_matches!(svc.resource_read_dynamic(&res.id), Some(_));
            }

            #[test]
            fn fails_if_owner_is_invalid() {
                let mut svc = SystemManagerInner::new();
                let res = CreateResource::default();

                svc.resource_create(res.clone()).unwrap_err();

                assert_eq!(svc.resources.len(), 0);
                assert_eq!(svc.resource_read(&res.id), None);
            }
        }
    }
}
