pub mod v1 {
    pub use super::containeros::system_manager::internal::server::v1::{
        ResourceCreateDynamicRequest,
        ResourceCreateDynamicResponse,
        ResourceReadRequest,
        ResourceReadResponse,
        system_manager_internal_service_server as svc,
    };
}

pub(crate) mod containeros {
    pub(crate) mod system_manager {
        pub(crate) mod shared {
            pub(crate) mod v1 {
                pub(crate) use cos_api_shared::proto::v1::*;
            }
        }

        pub(crate) mod internal {
            pub(crate) mod server {
                pub(crate) mod v1 {
                    #![allow(
                        clippy::all,
                        clippy::pedantic,
                        clippy::nursery,
                        clippy::restriction,
                        warnings,
                        unknown_lints
                    )]

                    tonic::include_proto!(
                        "containeros.system_manager.internal.server.v1"
                    );
                }
            }
        }
    }
}
