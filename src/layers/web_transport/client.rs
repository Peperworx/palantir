//! # Client Layer
//! 
//! Contains a webtransport client that implements layer. Can only communicate with the host.

use std::collections::HashMap;

use wtransport::{Endpoint, endpoint::endpoint_side::Client, ClientConfig};

pub struct WebTransportClient<NamespaceID: PartialEq + Hash> {
    /// The actual webtransport client
    client: Endpoint<Client>,
    /// The array map of namespace channels
    namespaces: HashMap<NamespaceID, ()>,
}

impl WebTransportClient {
    pub fn new() {
        let connection = Endpoint::client(ClientConfig::default()).unwrap()
            .connect("127.0.0.1:8092").await
            .unwrap();
    }
}