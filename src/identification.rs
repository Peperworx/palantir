//! # Identification
//! Contains structs and enums that are used for identifying peers on different layers.

use rand::Rng;
use sha2::Digest;
use serde::{Serialize, Deserialize};

/// # [`HostedPeerID`]
/// This enum is used to identify peers that are connected using layers that are split into host and client parts. (hosted layers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HostedPeerID {
    /// This variant identifies the peer as the host.
    Host,
    /// This variant identifies the peer as a client connected to the host.
    /// Clients are identified by a 256-bit ID, which is stored as two u128s.
    /// The actual value of these numbers are entirely layer-specific, however hosted layers that randomly
    /// generate IDs should use [`generate_random_id`] to generate them.
    Client([u128; 2]),
}

/// Generates a random 256-bit identifier, output as two u128s.
/// This function takes in a 128-bit namespace, which is used in the generation of the id.
/// Generated IDs are designed to have as low of a chance of collision as possible.
/// This is first done by generating a 256-bit number, and then hashing it with SHA-2 256.
/// The first 64 bits of the  id are the unix timestamp at which the id was generated.
/// This is to ensure that if any of the randomly generated values are the same, it must happen in the same
/// second for there to be a collision. This is followed by a 64-bit randomly generated number.
/// The next 128 bits are a UUIDv5, which is generated from the input namespace and a 128-bit randomly generated number,
/// that is then xored with the first 128 bits.
/// The entire id is then SHA-256 hashed so no information is leaked reguarding generation time or the namespace.
/// The output is then split into two u128s, which are returned.
pub fn generate_random_id(namespace: u128) -> [u128; 2] {
    // Get the RNG
    let mut rng = rand::thread_rng();


    // Get the current unix timestamp
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    // Generate a random 64-bit number
    let randnum = rng.gen::<u64>();

    // Combine the first 128 bits into a single 128-bit number
    let first = (timestamp as u128) << 64 | (randnum as u128);

    // xor a random u128 value with this number to get the name for the UUIDv5
    let name = first ^ rng.gen::<u128>();

    // Generate a UUIDv5 from the namespace and a random 128-bit number
    let new_uuid = uuid::Uuid::new_v5(&uuid::Uuid::from_u128(namespace), &name.to_le_bytes());

    // Append all of the values together as a [u8; 32]
    let mut id = [0u8; 32];
    let (left,right) = id.split_at_mut(16);
    left.copy_from_slice(&first.to_le_bytes());
    right.copy_from_slice(new_uuid.as_bytes());

    // SHA-2 256 hash them
    let mut hasher = sha2::Sha256::new();
    hasher.update(&id);
    let id = hasher.finalize();

    // Convert first 128 bits to a u128
    let first = u128::from_le_bytes(id[0..16].try_into().unwrap());
    let last = u128::from_le_bytes(id[16..].try_into().unwrap());

    [first, last]
}