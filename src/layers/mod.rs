//! # Layers




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

    /// The type of the peer's namespace identifiers
    type NSID;

    /// The message type sent by the peer
    type Message;

    
}