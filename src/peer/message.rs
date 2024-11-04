//! # Message
//! 
//! Contains the message type sent between peers when connecting.

use serde::{Deserialize, Serialize};
use slotmap::new_key_type;


/// # [`ActorID`]
/// A way to represent an actor over the network.
/// Can either be an actor's numerical ID, or a name for the actor.
#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum ActorID {
    ID(u64) = 0,
    Name(String) = 1,
}


new_key_type! {
    /// # [`RequestID`]
    /// The request ID type used to index responders in a slot map.
    pub struct RequestID;
}

/// # [`PeerMessage`]
/// The message type sent between peers.
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum PeerMessage {
    /// # [`PeerMessage::Initialize`]
    /// This is the first message sent over a channel by the initiating peer.
    /// It indicates the purpose of the channel.
    Initialize(ChannelPurpose) = 0,

    /// # [`PeerMessage::Handshake`]
    /// The handshake packet sent by each peer if the channel is initialized as a handshake channel.
    Handshake {
        /// The sending peer's name
        name: String,
    } = 1,

    /// # [`PeerMessage::HandshakeResponse`]
    /// The response to the handshake sent by each peer
    HandshakeResponse(HandshakeResponse) = 2,

    /// # [`PeerMessage::Request`]
    /// A request with arbitrary data that is sent over channels whose purpose is request/response.
    Request {
        /// This request's id
        request_id: RequestID,
        /// The request data
        body: Vec<u8>,
    } = 3,

    /// # [`PeerMessage::Response`]
    /// A response to a [`PeerMessage::Response`].
    Response {
        /// The ID of the request that this is a response to
        request_id: RequestID,
        /// The response data
        body: Vec<u8>,
    } = 4,
}


/// # [`ChannelPurpose`]
/// Indicates what a channel will be used for.
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelPurpose {
    /// # [`ChannelPurpose::Handshake`]
    /// Indicates that the channel will be used to conduct a single handshake.
    Handshake = 0,
    /// # [`ChannelPurpose::RequestResponse`]
    /// Indicates that the channel will be used as a request/response channel
    /// connected to a given actor id.
    RequestResponse(ActorID) = 1,
}

/// # [`HandshakeResponse`]
/// The different handshake outcomes that a peer could send to the other
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum HandshakeResponse {
    /// # [`HandshakeResponse::Ok`]
    /// The handshake finished successfully
    Ok = 0,
    /// # [`HandshakeResponse::NameTaken`]
    /// The handshake failed, as the peer's name was already taken
    NameTaken = 1,
}