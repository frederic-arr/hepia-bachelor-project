pub mod v1 {
    pub use super::containeros::system_manager::shared::v1::*;
}

pub(crate) mod containeros {
    pub(crate) mod system_manager {
        pub(crate) mod shared {
            pub(crate) mod v1 {
                #![allow(
                    clippy::all,
                    clippy::pedantic,
                    clippy::nursery,
                    clippy::restriction,
                    warnings,
                    unknown_lints
                )]

                tonic::include_proto!("containeros.system_manager.shared.v1");
            }
        }
    }
}
