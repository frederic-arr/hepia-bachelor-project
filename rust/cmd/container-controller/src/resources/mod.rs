mod image;
mod instance;
mod network;
mod runtime;

use std::sync::LazyLock;

use cos_proto_state_client::v1::StateServiceClient;
pub use image::*;
pub use instance::*;
pub use network::*;
pub use runtime::*;
use tonic::transport::{Channel, Endpoint};

pub static STATE_CLIENT: LazyLock<StateServiceClient<Channel>> =
    LazyLock::new(|| {
        StateServiceClient::new(
            Endpoint::from_static("http://127.0.0.1:50050").connect_lazy(),
        )
    });
