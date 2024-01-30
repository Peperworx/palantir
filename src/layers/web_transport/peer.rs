//! # Webtransport Peer
//! [`WTPeer`] provides a [`Peer`] implementation wrapping a webtransport [`Connection`]

use std::{collections::HashSet, marker::PhantomData, ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};
use wtransport::Connection;

use crate::{identification::HostedPeerID, layers::{Namespace, Peer}};

use super::{namespace::WTNamespace, WTConnectionError, WTLayerError, WTNamespaceID, WTStreamError};



pub struct WTPeer<P> {
    /// The connection
    pub(super) conn: Connection,
    /// The peer's id
    id: HostedPeerID,
    // The set of namespaces used
    namespaces: HashSet<WTNamespaceID>,
    _phantom: PhantomData<P>
}

impl<P> WTPeer<P> {
    pub fn new(conn: Connection, id: HostedPeerID) -> Self {
        Self {
            conn,
            id,
            namespaces: HashSet::new(),
            _phantom: Default::default()
        }
    }
}
#[derive(Serialize, Deserialize, PartialEq)]
enum WTPeerPacket {
    /// Sent by the client, triggers the initialization of a namespace.
    InitializeNamespace(WTNamespaceID),
    /// Response to [`InitializeNamespace`]. If true, the connection successfully initialized the namespace.
    /// If false, the namespace does not exist.
    /// Future versions of this response may change. 
    NamespaceInitResponse(bool),

}

impl<P: Serialize + for<'a> Deserialize<'a>> Peer for WTPeer<P> {
    type ID = HostedPeerID;

    type Error = WTLayerError;

    type Namespace = WTNamespace<P>;

    async fn open_namespace(&self, id: <Self::Namespace as crate::layers::Namespace>::ID) -> Result<WTNamespace<P>, WTLayerError> {
        
        // If the namespace is already used, then error
        if self.namespaces.contains(&id) {
            return Err(WTLayerError::NamespaceExists(id));
        }
        
        // Open a bidirectional channnel. Explicit into needed here due to nested errors.
        let conn = self.conn.open_bi().await
            .map_err(WTConnectionError::from)?
            .await.map_err(WTStreamError::from)?;


        // Open the namespace
        let mut ns = WTNamespace::<P>::new(conn, Some(id.clone()));

        // Initialize the namespace
        ns.send_raw(bincode::serialize(&WTPeerPacket::InitializeNamespace(id))?).await?;

        // Receive the response
        let res: WTPeerPacket = bincode::deserialize(&ns.recv_raw().await?)?;

        // If successful, return the namespace. Otherwise, return an error
        if res == WTPeerPacket::NamespaceInitResponse(true) {
            Ok(ns)
        } else {
            Err(WTLayerError::NamespaceDenied)
        }
    }

    async fn wait_namespace(&self) -> Result<Self::Namespace, Self::Error> {

        // Wait for a new bidirectional channel
        let conn = self.conn.accept_bi().await
            .map_err(WTConnectionError::from)?;

        // Open the namespace
        let mut ns = WTNamespace::<P>::new(conn, None);

        // Wait for a namespace initialization
        let WTPeerPacket::InitializeNamespace(nsid) = bincode::deserialize(&ns.recv_raw().await?)? else {
            return Err(WTLayerError::InvalidNSPacket);
        };

        // If the namespace is already used, then send failure message and error
        if self.namespaces.contains(&nsid) {
            ns.send_raw(bincode::serialize(&WTPeerPacket::NamespaceInitResponse(false))?).await?;
            return Err(WTLayerError::NamespaceExists(nsid));
        }

        // Update the namespace id
        ns.set_id(nsid);

        // Respond to the initialization
        ns.send_raw(bincode::serialize(&WTPeerPacket::NamespaceInitResponse(true))?).await?;

        // Return the namespace
        Ok(ns)
    }

    fn get_id(&self) -> Self::ID {
        self.id
    }

    
}

impl<P: Serialize + for<'a> Deserialize<'a>> Peer for Arc<WTPeer<P>> {
    type ID = HostedPeerID;

    type Error = WTLayerError;

    type Namespace = WTNamespace<P>;

    async fn open_namespace(&self, id: <Self::Namespace as Namespace>::ID) -> Result<Self::Namespace, Self::Error> {
        self.deref().open_namespace(id).await
    }

    async fn wait_namespace(&self) -> Result<Self::Namespace, Self::Error> {
        self.deref().wait_namespace().await
    }

    fn get_id(&self) -> Self::ID {
        self.deref().get_id()
    }
}