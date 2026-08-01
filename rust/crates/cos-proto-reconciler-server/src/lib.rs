#![feature(decl_macro)]

pub mod v1 {
    #![allow(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::restriction,
        warnings,
        unknown_lints
    )]

    mod _proto {
        tonic::include_proto!("containeros.reconciler.v1");
    }

    pub use self::_proto::reconciler_service_server::*;
}

pub macro router {
    ($res:ident, $maybe:ident, $resource:ident, $reconciler:expr) => {{
        let spec = ::serde_json::from_value($res.spec.clone())
            .map_err(|err| ::tonic::Status::from_error(err.into()))?;

        let maybe_resource = $maybe
            .map(|v| {
                Ok::<_, ::anyhow::Error>($resource {
                    id: v.id,
                    phase: v.phase,
                    status: v.status,
                    spec: ::serde_json::from_value(v.spec)?,
                    derived_spec: ::serde_json::from_value(v.derived_spec)?,
                    state: v.state.map(::serde_json::from_value).transpose()?,
                    children: v.children,
                    dependencies: v.dependencies,
                    dependents: v.dependents,
                })
            })
            .transpose()
            .map_err(|err| ::tonic::Status::from_error(err.into()))?;

        ($reconciler, spec, maybe_resource)
    }},

    ($res:ident, $resource:ident, $reconciler:expr) => {{
        let resource = $resource {
            id: $res.id,
            phase: $res.phase,
            status: $res.status,
            spec: ::serde_json::from_value($res.spec)
                .map_err(|err| ::tonic::Status::from_error(err.into()))?,
            derived_spec: ::serde_json::from_value($res.derived_spec)
                .map_err(|err| ::tonic::Status::from_error(err.into()))?,
            state: $res
                .state
                .map(::serde_json::from_value)
                .transpose()
                .map_err(|err| ::tonic::Status::from_error(err.into()))?,
            children: $res.children,
            dependencies: $res.dependencies,
            dependents: $res.dependents,
        };

        ($reconciler, resource)
    }}
}

pub macro validate($res:ident, $maybe:ident, $resource:ident, $reconciler:expr) {
    let (reconciler, spec, maybe_resource) =
        router!($res, $maybe, $resource, $reconciler);
    let response = reconciler
        .validate($res.id.key().clone(), spec, maybe_resource)
        .await
        .map_err(|err| ::tonic::Status::from_error(err.into()))?;

    return Ok(::tonic::Response::new(
        ::cos_proto_reconciler::v1::ValidateResponse {
            raw: ::serde_json::to_vec(&response)
                .map_err(|err| ::tonic::Status::from_error(err.into()))?,
        },
    ));
}

pub macro reconcile($res:ident, $resource:ident, $reconciler:expr) {
    let (reconciler, resource) = router!($res, $resource, $reconciler);
    let response = reconciler
        .reconcile(resource)
        .await
        .map_err(|err| ::tonic::Status::from_error(err.into()))?;

    return Ok(::tonic::Response::new(
        ::cos_proto_reconciler::v1::ReconcileResponse {
            raw: ::serde_json::to_vec(&response)
                .map_err(|err| ::tonic::Status::from_error(err.into()))?,
        },
    ));
}
