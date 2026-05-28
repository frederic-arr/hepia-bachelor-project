use bollard::Docker;
use cos_api_reconciler::proto::v1::ReconcileDynamicResourceResponse;
use cos_api_reconciler_server::{Reconcilable, ReconcilableDriver};
use cos_api_shared::{
    DynamicResource,
    Identity,
    Resource,
    ResourceSpec,
    Specification,
    State,
    UserConfigResource,
};
use derive_builder::Builder;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContainerSpec {
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Deserialize, Serialize)]
#[builder(pattern = "mutable")]
pub struct ContainerState {
    pub id: String,
    pub running: bool,
    pub paused: bool,
}

impl Specification for ContainerSpec {
    type State = ContainerState;

    const SCHEMA: &str = ".containeros.containers.container-spec";
}

impl State for ContainerState {}

impl ContainerState {
    pub async fn refresh(
        id: Identity,
        spec: &ContainerSpec,
        docker: &mut Docker,
    ) -> Option<Self> {
        let mut state = ContainerStateBuilder::default();

        let inspect = docker.inspect_container(id.name(), None).await.unwrap();
        if let Some(id) = inspect.id {
            state.id(id);
        }

        let container_state = inspect.state.unwrap();
        if let Some(running) = container_state.running {
            state.running(running);
        }

        if let Some(paused) = container_state.paused {
            state.paused(paused);
        }

        state.build().map(Some).unwrap()
    }
}
