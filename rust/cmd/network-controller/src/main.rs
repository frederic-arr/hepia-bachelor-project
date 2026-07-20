use anyhow::Result;
use cos_proto_reconciler::v1::{
    ReconcileRequest,
    ReconcileResponse,
    ValidateRequest,
    ValidateResponse,
};
use cos_proto_reconciler::{Identity, Resource, SubResourceCreate};
use cos_proto_reconciler_server::v1::{
    ReconcilerService,
    ReconcilerServiceServer,
};
use network_controller::{
    AddressReconciler,
    AddressResource,
    DhcpReconciler,
    DhcpResource,
    DnsReconciler,
    DnsResource,
    LinkReconciler,
    LinkResource,
    RouteReconciler,
    RouteResource,
};
use rtnetlink::new_connection;
use serde_json::Value;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct Reconciler;

macro_rules! router {
    ($res:ident, $maybe:ident, $resource:ident, $reconciler:expr) => {{
        let spec = serde_json::from_value($res.spec.clone())
            .map_err(|err| Status::from_error(err.into()))?;

        let maybe_resource = $maybe
            .map(|v| {
                Ok::<_, anyhow::Error>($resource {
                    id: v.id,
                    phase: v.phase,
                    status: v.status,
                    spec: serde_json::from_value(v.spec)?,
                    derived_spec: serde_json::from_value(v.derived_spec)?,
                    state: v.state.map(serde_json::from_value).transpose()?,
                    children: v.children,
                    dependencies: v.dependencies,
                    dependents: v.dependents,
                })
            })
            .transpose()
            .map_err(|err| Status::from_error(err.into()))?;

        ($reconciler, spec, maybe_resource)
    }};
    ($res:ident, $resource:ident, $reconciler:expr) => {{
        let resource = $resource {
            id: $res.id,
            phase: $res.phase,
            status: $res.status,
            spec: serde_json::from_value($res.spec)
                .map_err(|err| Status::from_error(err.into()))?,
            derived_spec: serde_json::from_value($res.derived_spec)
                .map_err(|err| Status::from_error(err.into()))?,
            state: $res
                .state
                .map(serde_json::from_value)
                .transpose()
                .map_err(|err| Status::from_error(err.into()))?,
            children: $res.children,
            dependencies: $res.dependencies,
            dependents: $res.dependents,
        };

        ($reconciler, resource)
    }};
}

macro_rules! validate {
    ($res:ident, $maybe:ident, $resource:ident, $reconciler:expr) => {
        let (reconciler, spec, maybe_resource) =
            router!($res, $maybe, $resource, $reconciler);
        let response = reconciler
            .validate(spec, maybe_resource)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        return Ok(Response::new(ValidateResponse {
            raw: serde_json::to_vec(&response)
                .map_err(|err| Status::from_error(err.into()))?,
        }));
    };
}

macro_rules! reconcile {
    ($res:ident, $resource:ident, $reconciler:expr) => {
        let (reconciler, resource) = router!($res, $resource, $reconciler);
        let response = reconciler
            .reconcile(resource)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        return Ok(Response::new(ReconcileResponse {
            raw: serde_json::to_vec(&response)
                .map_err(|err| Status::from_error(err.into()))?,
        }));
    };
}

#[tonic::async_trait]
impl ReconcilerService for Reconciler {
    async fn validate(
        &self,
        request: Request<ValidateRequest>,
    ) -> Result<Response<ValidateResponse>, Status> {
        let req = request.into_inner();
        let (resource, maybe_resource): (
            SubResourceCreate<Value>,
            Option<Resource<Value, Value, Value>>,
        ) = serde_json::from_slice(&req.raw)
            .map_err(|err| Status::from_error(err.into()))?;

        let key = resource.id.key();
        match key.schema.as_ref() {
            "network:dns" => {
                validate!(
                    resource,
                    maybe_resource,
                    DnsResource,
                    DnsReconciler::new()
                );
            }
            "network:route" => {
                validate!(resource, maybe_resource, RouteResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    RouteReconciler::new_with(handle)
                });
            }
            "network:address" => {
                validate!(resource, maybe_resource, AddressResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    AddressReconciler::new_with(handle)
                });
            }
            "network:link" => {
                validate!(resource, maybe_resource, LinkResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    LinkReconciler::new_with(handle)
                });
            }
            "network:dhcp" => {
                validate!(resource, maybe_resource, DhcpResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    DhcpReconciler::new_with(handle)
                });
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }

    async fn reconcile(
        &self,
        request: Request<ReconcileRequest>,
    ) -> Result<Response<ReconcileResponse>, Status> {
        let req = request.into_inner();
        let resource: Resource<Value, Value, Value> =
            serde_json::from_slice(&req.raw)
                .map_err(|err| Status::from_error(err.into()))?;

        let key = resource.id.key();
        match key.schema.as_ref() {
            "network:dns" => {
                reconcile!(resource, DnsResource, DnsReconciler::new());
            }
            "network:route" => {
                reconcile!(resource, RouteResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    RouteReconciler::new_with(handle)
                });
            }
            "network:address" => {
                reconcile!(resource, AddressResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    AddressReconciler::new_with(handle)
                });
            }
            "network:link" => {
                reconcile!(resource, LinkResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    LinkReconciler::new_with(handle)
                });
            }
            "network:dhcp" => {
                reconcile!(resource, DhcpResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    DhcpReconciler::new_with(handle)
                });
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }
}

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let addr = "[::1]:50052".parse()?;
    let reconciler = Reconciler;

    tracing::info!("network controller listening on {addr}");

    Server::builder()
        .add_service(ReconcilerServiceServer::new(reconciler))
        .serve(addr)
        .await?;

    Ok(())
}
