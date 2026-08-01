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
        tonic::include_proto!("containeros.state.v1");
    }

    pub use self::_proto::state_service_client::*;
}
