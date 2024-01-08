//! # Client Layer
//! 
//! Contains a webtransport client that implements layer. Can only communicate with the host.


use wtransport::Connection;
use wtransport::endpoint::ConnectOptions;
use wtransport::{Endpoint, endpoint::endpoint_side::Client, ClientConfig};

use crate::layers::web_transport::{WebTransportPacket, WebTransportCodec, WebTransportNamespace};

use super::{WebTransportLayerError, WebTransportNamespaceID};

pub struct WebTransportClient {
    /// The connection to the host
    connection: Connection,
}

impl WebTransportClient {
    /// Creates a new [`WebTransportClient`], connecting to the given host connection options.
    /// Does not initiate any namespaces.
    pub async fn connect(options: ConnectOptions) -> Result<Self, WebTransportLayerError> {
        // Create the client endpoint
        let client = Endpoint::client(ClientConfig::default())?;

        // Connect to the host
        let connection = client.connect(options).await?;

        // Create the client
        let mut client = Self {
            connection
        };

        // Open the core namespace
        let ns = client.open_namespace(WebTransportNamespaceID::Core).await?;

        
        
        Ok(client)
    }

    /// Opens a bidirectional connnection to the specified namespace, or to the core namespace if no
    /// namespace value is supplied
    async fn open_namespace(&self, namespace: WebTransportNamespaceID) -> Result<WebTransportNamespace, WebTransportLayerError> {

        // Open a bidirectional channnel
        let (send, recv) = self.connection.open_bi().await?.await?;
        
        // Wrap in the codec.
        let mut codec = WebTransportCodec::new(send, recv);

        // Trigger opening the namespace
        codec.send(&WebTransportPacket::InitializeNamespace(namespace.clone())).await?;

        // Read the response. If it is wrong, then generate an error
        if codec.recv().await? != WebTransportPacket::NamespaceInitResponse(true) {
            return Err(WebTransportLayerError::NamespaceOpenError);
        }

        // The namespace has been opened, so we can wrap the codec with a [`WebTransportNamespace`]
        let ns = WebTransportNamespace(codec, namespace);

        Ok(ns)
    }
}


