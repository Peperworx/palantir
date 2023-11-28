//! # Palantir

use serde::{Serialize, Deserialize};
use sha2::Digest;

pub mod transport;

/// # [`PeerId`]
/// Stores a peer's ID as a 512 bit little-endian encoded number.
/// Generally, the returned peer id's should be generated from a cryptographically-secure hashing algorithm,
/// preferrably by public key.
/// This should be created deterministically. If a peer connects multiple times to the transport, the same ID should be issued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId(pub [u8; 64]);

impl From<ecdsa::VerifyingKey<p384::NistP384>> for PeerId {
    fn from(value: ecdsa::VerifyingKey<p384::NistP384>) -> Self {
        // Get the SHA2-512 hash of the key represented in SEC1 format
        let mut hasher = sha2::Sha512::new();
        hasher.update(value.to_sec1_bytes());

        // Finalize into a known-sized slice
        let mut hashed = [0u8; 64];
        hasher.finalize_into((&mut hashed).into());

        // Wrap in peer id
        PeerId(hashed)
    }
}

/// When converted to a string, the PeerId will be converted as Big-Endian hex bytes
impl ToString for PeerId {
    fn to_string(&self) -> String {
        let mut out = String::new();
        for b in self.0 {
            out.push_str(&format!("{:x}", b));
        }
        out
    }
}

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
