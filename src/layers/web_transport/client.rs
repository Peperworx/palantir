//! # Client Layer
//! 
//! Contains a webtransport client that implements layer. Can only communicate with the host.


use std::sync::Arc;

use serde::{Deserialize, Serialize};

use wtransport::endpoint::ConnectOptions;
use wtransport::{Endpoint, ClientConfig};

use crate::identification::HostedPeerID;

use super::peer::WTPeer;
use crate::layers::Layer;
use super::{WTConnectionError, WTLayerError};


pub struct WTClient<P> {
    /// The peer connection to the host
    peer: Arc<WTPeer<P>>,

}

impl<P: Serialize + for<'a> Deserialize<'a>> WTClient<P> {
    /// Creates a new [`WTClient`], connecting to the given host connection options.
    /// Does not initiate any namespaces.
    pub async fn connect(options: ConnectOptions) -> Result<Self, WTLayerError> {
        // Create the client endpoint
        let client = Endpoint::client(ClientConfig::default())?;

        // Connect to the host
        let connection = client.connect(options).await
            .map_err(WTConnectionError::from)?;

        // Create the peer
        let peer = Arc::new(WTPeer::<P>::new(connection, HostedPeerID::Host));

        Ok(Self {
            peer
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