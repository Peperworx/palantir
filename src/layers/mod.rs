//! # Layers

use std::error::Error;

use serde::{Deserialize, Serialize};




pub mod web_transport;

/// # [`Layer`]
/// A [`Layer`] allows retreiving connections to peers.
pub trait Layer: Send {
    
    /// The peer type used by this layer
    type Peer: Peer;

    /// Get a connected peer, returning None if the peer does not exist
    fn get_peer(&self, id: <Self::Peer as Peer>::ID) -> Option<Self::Peer>;
}

pub trait Peer {

    /// The type of the peer's identifier
    type ID;
    
    /// The error type used by the peer
    type Error: Error;

    /// The namespace type used by the peer
    type Namespace: Namespace;

    /// Open a bidirectional connection with the peer over a namespace
    async fn open_namespace(&self, id: <Self::Namespace as Namespace>::ID) -> Result<Self::Namespace, Self::Error>;

    /// Wait for a namespace to be initiated with this peer
    async fn wait_namespace(&self) -> Result<Self::Namespace, Self::Error>;
}

pub trait Namespace {

    /// The type of the namespace's identifier
    type ID;

    /// The packet type
    type Packet: Serialize + for<'a> Deserialize<'a>;

    /// The error received when transmitting a packet
    type Error: Error;

    /// Send a packet over the namespace
    async fn send(&mut self, packet: &Self::Packet) -> Result<(), Self::Error>;

    /// Wait for a packet to be received
    async fn recv(&mut self) -> Result<Self::Packet, Self::Error>;

    /// Get the namespace's ID. Return None if not initializes
    fn get_id(&self) -> Option<Self::ID>;
}