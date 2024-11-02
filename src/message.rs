//! # Message
//! The data packet type that palantir sends over the wire, serialized using [`postcard`].

use serde::{Deserialize, Serialize};



/// # [`PalantirMessage`]
/// The message structure that palantir sends over the wire, serialized using [`postcard`]
#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum PalantirMessage<V> {
    /// # [`PalantirMessage::ClientHandshake`]
    /// The handshake sent from a client to the server when the connection is opened.
    ClientHandshake {
        /// The magic string. Should always be "PALANTIR"
        magic: [char; 8],
        /// The name of the connecting peer
        name: String,
        /// The arbitrary validation sent by the client
        validation: V,
    } = 0,
    /// # [`PalantirMessage::ServerResponse`]
    /// The handshake sent from the server to a client after the client handshake.
    ServerResponse {
        /// The magic string. Should always be "PALANTIR"
        magic: [char; 8],
        /// The name of the server
        name: String,
    } = 1,
    /// # [`PalantirMessage::ClientResponse`]
    /// An empty packet sent by the client to verify it has received the server's response and everything is OK.
    ClientResponse = 2,
    /// # [`PalantirMessage::ValidationFailed`]
    /// Sent by the server to indicate to the client that the validation failed.
    ValidationFailed = 3,
    /// # [`PalantirMessage::MalformedData`]
    /// Sent by a peer when it receives invalid data
    MalformedData = 4,
    /// # [`PalantirMessage::UnexpectedPacket`]
    /// Sent by a peer when it receives an unexpected packet
    UnexpectedPacket = 5,
    /// # [`PalantirMessage::Request`]
    /// A request containing arbitrary data from a client
    Request {
        /// The request id
        id: u64,
        /// The request data
        data: Vec<u8>,
    } = 6,
    /// # [`PalantirMessage::Response`]
    /// A response from the server, containing arbitrary data
    Response {
        /// The ID of the request we are responding to.
        id: u64,
        /// The response data
        data: Vec<u8>,
    } = 7,
}