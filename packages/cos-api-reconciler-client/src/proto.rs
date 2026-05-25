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

    pub use self::_proto::reconciler_service_client::*;
}
