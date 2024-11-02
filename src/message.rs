//! # Message
//! The data packet type that palantir sends over the wire, serialized using [`postcard`].

use serde::{Deserialize, Serialize};


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
pub enum PalantirMessage<V> {


    /// # [`PalantirMessage::ClientHandshake`]
    /// The handshake sent from a client to the server when the connection is opened.
    /// Indicates that the connection is a handshake connection and that it should be used for
    /// validation only, then closed.
    ClientHandshake {
        /// The magic string. Should always be "PALANTIR"
        magic: [char; 8],
        /// The name of the connecting peer
        name: String,
        /// The arbitrary validation sent by the client
        validation: V,
    },
    /// # [`PalantirMessage::ServerResponse`]
    /// The handshake sent from the server to a client after the client handshake.
    ServerResponse {
        /// The magic string. Should always be "PALANTIR"
        magic: [char; 8],
        /// The name of the server
        name: String,
    },
    /// # [`PalantirMessage::ClientResponse`]
    /// Sent by the client to acknowledge the handshake's completion
    ClientResponse,


    /// # [`PalantirMessage::Connect`]
    /// Indicates to the client that it wants to send messages to the given actor.
    Connect(ActorID),
    /// # [`PalantirMessage::ActorDoesntExist`]
    /// Sent when the actor requested by the client doesn't exist.
    ActorDoesntExist,
    /// # [`PalantirMessage::Request`]
    /// A request containing arbitrary data from a client
    Request {
        /// The request id
        id: u64,
        /// The request data
        data: Vec<u8>,
    },
    /// # [`PalantirMessage::Response`]
    /// A response from the server, containing arbitrary data
    Response {
        /// The ID of the request we are responding to.
        id: u64,
        /// The response data
        data: Vec<u8>,
    },
    

    /// # [`PalantirMessage::MalformedData`]
    /// Sent by a peer when it receives invalid data
    MalformedData,
    /// # [`PalantirMessage::UnexpectedPacket`]
    /// Sent by a peer when it receives an unexpected packet
    UnexpectedPacket,

}