//! # Palantir


pub mod transport;

/// # [`PeerId`]
/// Stores a peer's ID as a 256 bit little-endian encoded number.
/// Generally, the returned peer id's should be generated from a cryptographically-secure hashing algorithm,
/// preferrably by public key.
/// This should be created deterministically. If a peer connects multiple times to the transport, the same ID should be issued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId(pub [u8; 32]);

/// # [`ConnectionId`]
/// Stores a connection's ID as a 128 bit little-endian encoded number, as well as the PeerId with which the connection is associated.
/// This should generally be a UUID generated at connection-initialization time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConnectionId(pub PeerId, pub [u8; 16]);