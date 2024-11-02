//! # Message
//! The data packet type that palantir sends over the wire, serialized using [`postcard`].

use serde::{Deserialize, Serialize};

use crate::validation::Validator;


/// # [`Side`]
/// Indicates which end of a connection the given method is executing on:
/// either the initiator or the acceptor.
/// (Client or server, but this distinction is important)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Side {
    Initiator,
    Acceptor
}

/// # [`ActorID`]
/// A way to represent an actor over the network.
/// Can either be an actor's numerical ID, or a name for the actor.
#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum ActorID {
    ID(u64) = 0,
    Name(String) = 1,
}

/// # [`PalantirMessage`]
/// The message structure that palantir sends over the wire, serialized using [`postcard`]
#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum PalantirMessage<V: Validator> {

    /// # [`PalantirMessage::ClientInitiation`]
    /// Sent by the client to the server when it initiates a connection.
    /// Contains the client's name, and a magic string that should always be "PALANTIR"
    ClientInitiation {
        magic: String,
        name: String,
    } = 0,
    /// # [`PalantirMessage::ServerResponse`]
    /// Send by the server to the client in response to a [`PalantirMessage::ClientInitiation`].
    /// Contains the server's name and the magic string ("PALANTIR").
    ServerResponse {
        magic: String,
        name: String,
    } = 1,

    /// # [`PalantirMessage::ValidatorPacket`]
    /// Arbitrary packet exchanged by the validator after [`PalantirMessage::ServerResponse`]
    /// but before [`PalantirMessage::HandshakeCompleted`].
    ValidatorPacket(V::Packet) = 2,
    
    /// # [`PalantirMessage::HandshakeCompleted`]
    /// Send by the client to the server to indicate that the handshake was completed.
    HandshakeCompleted = 3,

    /// # [`PalantirMessage::Request`]
    /// Indicates a request with the given ID and arbitrary data.
    Request {
        /// The request's id.
        id: u32,
        /// The request's data
        data: Vec<u8>,
    } = 4,

    /// # [`PalantirMessage::Response`]
    /// Contains the response data to a request with the given ID.
    Response {
        /// The ID of the request this response is to
        id: u32,
        /// The response data
        data: Vec<u8>
    } = 5,

    /// # [`PalantirMessage::NameTaken`]
    /// Sent by either peer to the other during the handshake
    /// to indicate that the given name was taken.
    /// The connection is then terminated.
    NameTaken,

    /// # [`PalantirMessage::InvalidMagic`]
    /// Sent by either peer to the other during the handshake
    /// to indicate that the given magic value was invalid.
    /// The connection is then terminated.
    InvalidMagic,

    /// # [`PalantirMessage::UnexpectedPacket`]
    /// Sent by either peer to the other to indicate that
    /// an unexpected packet was received.
    /// The connection is then terminated.
    UnexpectedPacket,

    /// # [`PalantirMessage::MalformedData`]
    /// Sent by either peer to the other to indicate
    /// that the data sent was malformed and unable
    /// to be deserialized.
    MalformedData,
}