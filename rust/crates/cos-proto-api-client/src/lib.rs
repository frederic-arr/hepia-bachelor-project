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
        tonic::include_proto!("containeros.api.v1");
    }

    pub use self::_proto::api_service_client::*;
}
