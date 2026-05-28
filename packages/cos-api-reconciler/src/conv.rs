use cos_api_shared::Specification;

use crate::model::*;
use crate::proto::v1;

impl<T> From<v1::CreateDynamicResourceRequest>
    for CreateDynamicResourceRequest<T>
where
    T: Specification,
{
    fn from(value: v1::CreateDynamicResourceRequest) -> Self {
        let id = value.id.unwrap();
        Self {
            id: Identity::new(id.schema, id.name),
            spec: rmp_serde::from_slice(&value.spec).unwrap(),
        }
    }
}

impl<T> From<v1::ReconcileDynamicResourceRequest>
    for ReconcileDynamicResourceRequest<T>
where
    T: Specification,
{
    fn from(value: v1::ReconcileDynamicResourceRequest) -> Self {
        let id = value.id.unwrap();
        let state = match value.state.unwrap() {
            v1::reconcile_dynamic_resource_request::State::Ready(
                v1::reconcile_dynamic_resource_request::StateReady { state },
            ) => state,
            v1::reconcile_dynamic_resource_request::State::Error(
                v1::reconcile_dynamic_resource_request::StateError {
                    state,
                    ..
                },
            ) => state,
        };

        Self {
            id: Identity::new(id.schema, id.name),
            spec: rmp_serde::from_slice(&value.spec).unwrap(),
            state: rmp_serde::from_slice(&state).unwrap(),
        }
    }
}
