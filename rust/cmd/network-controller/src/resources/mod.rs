mod address;
mod dhcp;
mod dns;
mod link;
mod ntp;
mod route;

use std::sync::LazyLock;

pub use address::*;
use cos_proto_state_client::v1::StateServiceClient;
pub use dhcp::*;
pub use dns::*;
pub use link::*;
pub use ntp::*;
pub use route::*;
use tonic::transport::{Channel, Endpoint};

pub static STATE_CLIENT: LazyLock<StateServiceClient<Channel>> =
    LazyLock::new(|| {
        StateServiceClient::new(
            Endpoint::from_static("http://127.0.0.1:50050").connect_lazy(),
        )
    });
