//! # Client Layer
//! 
//! Contains a webtransport client that implements layer. Can only communicate with the host.


use serde::{Deserialize, Serialize};
use wtransport::Connection;
use wtransport::endpoint::ConnectOptions;
use wtransport::{Endpoint, endpoint::endpoint_side::Client, ClientConfig};

use crate::identification::HostedPeerID;

use super::namespace::WTNamespace;
use super::peer::WTPeer;
use crate::layers::Peer;
use super::{WTNamespaceID, WebTransportLayerError};


pub struct WTClient<P> {
    /// The peer connection to the host
    peer: WTPeer<P>,
    /// The core namespace
    core: WTNamespace<P>,
}

impl<P: Serialize + for<'a> Deserialize<'a>> WTClient<P> {
    /// Creates a new [`WTClient`], connecting to the given host connection options.
    /// Does not initiate any namespaces.
    pub async fn connect(options: ConnectOptions) -> Result<Self, WebTransportLayerError> {
        // Create the client endpoint
        let client = Endpoint::client(ClientConfig::default())?;

        // Connect to the host
        let connection = client.connect(options).await?;

        // Create the peer
        let peer = WTPeer::<P>::new(connection, HostedPeerID::Host);

        // Open the core namespace
        let core = peer.open_namespace(WTNamespaceID::Core).await?;
        
        Ok(Self {
            peer,
            core,
        })
    }

}


