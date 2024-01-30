//! # Client Layer
//! 
//! Contains a webtransport client that implements layer. Can only communicate with the host.


use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wtransport::Connection;
use wtransport::endpoint::ConnectOptions;
use wtransport::{Endpoint, endpoint::endpoint_side::Client, ClientConfig};

use crate::identification::HostedPeerID;

use super::namespace::WTNamespace;
use super::peer::WTPeer;
use crate::layers::{Layer, Peer};
use super::{WTNamespaceID, WebTransportLayerError};


pub struct WTClient<P> {
    /// The peer connection to the host
    peer: Arc<WTPeer<P>>,
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
        let peer = Arc::new(WTPeer::<P>::new(connection, HostedPeerID::Host));

        // Open the core namespace
        let core = peer.open_namespace(WTNamespaceID::Core).await?;
        
        Ok(Self {
            peer,
            core,
        })
    }

}


impl<P: Serialize + for<'a> Deserialize<'a> + Send + Sync> Layer for WTClient<P> {
    type Peer = Arc<WTPeer<P>>;

    fn get_peer(&self, id: HostedPeerID) -> Option<Self::Peer> {
        if id == HostedPeerID::Host {
            Some(self.peer.clone())
        } else {
            None
        }
    }
}