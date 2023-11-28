//! # Palantir

use serde::{Serialize, Deserialize};

pub mod transport;

/// # [`PeerId`]
/// Stores a peer's ID as a 512 bit little-endian encoded number.
/// Generally, the returned peer id's should be generated from a cryptographically-secure hashing algorithm,
/// preferrably by public key.
/// This should be created deterministically. If a peer connects multiple times to the transport, the same ID should be issued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId(pub [u8; 64]);

/// # [`ConnectionId`]
/// Stores a connection's ID as a 128 bit little-endian encoded number, as well as the PeerId with which the connection is associated.
/// This should generally be a UUID generated at connection-initialization time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConnectionId(pub PeerId, pub [u8; 16]);


/// # [`Message`]
/// Type implemented for any message sent over a Palantir network.
pub trait Message: Serialize + for<'a> Deserialize<'a> {
    type Response: Serialize + for<'a> Deserialize<'a>;
}
