//! # Message
//! 
//! Contains the message type sent between peers when connecting.

use serde::{Deserialize, Serialize};


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

    /// # [`PeerMessage::Request`]
    /// A request with arbitrary data that is sent over channels whose purpose is request/response.
    Request = 2,

    /// # [`PeerMessage::Response`]
    /// A response to a [`PeerMessage::Response`].
    Response = 3,
}


/// # [`ChannelPurpose`]
/// Indicates what a channel will be used for.
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelPurpose {
    /// # [`ChannelPurpose::Handshake`]
    /// Indicates that the channel will be used to conduct a single handshake.
    Handshake = 0,
}