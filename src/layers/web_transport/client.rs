//! # Client Layer
//! 
//! Contains a webtransport client that implements layer. Can only communicate with the host.

use wtransport::{Endpoint, endpoint::endpoint_side::Client};

pub struct WebTransportClient {
    /// The actual webtransport client
    client: Endpoint<Client>,
}