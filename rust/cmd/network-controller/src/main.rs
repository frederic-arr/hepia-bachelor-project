use anyhow::Result;
use cos_proto_reconciler::v1::{
    ReconcileRequest,
    ReconcileResponse,
    ValidateRequest,
    ValidateResponse,
};
use cos_proto_reconciler::{Resource, SubResourceCreate};
use cos_proto_reconciler_server::v1::{
    ReconcilerService,
    ReconcilerServiceServer,
};
use cos_proto_reconciler_server::{reconcile, validate};
use network_controller::{
    AddressReconciler,
    AddressResource,
    DhcpReconciler,
    DhcpResource,
    DnsReconciler,
    DnsResource,
    LinkReconciler,
    LinkResource,
    NtpReconciler,
    NtpResource,
    RouteReconciler,
    RouteResource,
};
use rtnetlink::new_connection;
use rustix::thread::{CapabilitySet, CapabilitySets, set_capabilities};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::runtime::{Builder, LocalOptions};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct Reconciler;

const CAPS: CapabilitySet = CapabilitySet::NET_ADMIN
    .union(CapabilitySet::NET_RAW)
    .union(CapabilitySet::SYS_TIME)
    .union(CapabilitySet::NET_BIND_SERVICE);

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
            "network:ntp" => {
                validate!(
                    resource,
                    maybe_resource,
                    NtpResource,
                    NtpReconciler::new()
                );
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }

    async fn reconcile(
        &self,
        request: Request<ReconcileRequest>,
    ) -> Result<Response<ReconcileResponse>, Status> {
        set_capabilities(
            None,
            CapabilitySets {
                effective: CAPS,
                permitted: CAPS,
                inheritable: CAPS,
            },
        )
        .map_err(|err| Status::from_error(err.into()))?;

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
            "network:ntp" => {
                reconcile!(resource, NtpResource, NtpReconciler::new());
            }
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }
}

fn main() -> Result<()> {
    set_capabilities(
        None,
        CapabilitySets {
            effective: CapabilitySet::empty(),
            permitted: CAPS,
            inheritable: CAPS,
        },
    )?;

    Builder::new_current_thread()
        .enable_all()
        .build_local(LocalOptions::default())?
        .block_on(async_main())
}

async fn async_main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let addr = "127.0.0.1:50052";
    let reconciler = Reconciler;

    let listener = TcpListener::bind(addr).await?;
    let incoming = TcpListenerStream::new(listener);
    tracing::info!("network controller listening on {addr}");

    Server::builder()
        .add_service(ReconcilerServiceServer::new(reconciler))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}
