//! # Transports
//! Palantir does not restrict itself to a particular protocol while communicating between peers.
//! The [`Transport`] trait will help to abstract over different protocols.

use crate::{PeerId, ConnectionId};


/// # [`Transport`]
/// Abstracts over a protocol or suite of protocols used for communication between peers.
#[async_trait::async_trait]
pub trait Transport {

    /// Retrieves every known peer id.
    fn get_peers(&self) -> Vec<PeerId>;

    /// Waits until new peer(s) are available. If multiple peers become available at the same time,
    /// multiple peers will be returned.
    async fn wait_for_new_peer(&self) -> Vec<PeerId>;

    /// Lists every connection for a given peer
    fn get_connections_by_peer(&self, peer: PeerId) -> Vec<ConnectionId>;

    /// Waits for a new connection, returning the ConnectionId
    async fn wait_for_new_connection(&self) -> Vec<PeerId>;

    /// Forcefully drops a peer and every connection they have made.
    async fn drop_peer(&self, peer: PeerId, reason: u16);

    /// Forcefully drops a single connection for a peer.
    async fn drop_connection(&self, connection: ConnectionId, reason: u16);
}