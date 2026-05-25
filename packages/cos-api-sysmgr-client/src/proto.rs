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
        tonic::include_proto!(
            "containeros.system_manager.v1"
        );
    }

    pub use self::_proto::system_manager_service_client::*;
}
