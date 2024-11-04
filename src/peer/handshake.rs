use std::sync::Arc;

use wtransport::Connection;

use crate::{frame::Framed, peer::message::{ChannelPurpose, HandshakeResponse, PeerMessage}};

use super::{error::HandshakeError, Peer};




/// # [`handshake`]
/// Runs a handshake over a given channel, returning either the peer's name if the handshake succeeds,
/// or an error if not.
pub(crate) async fn handshake<V, H>(peer: Arc<Peer<V, H>>, connection: Connection, is_server: bool) -> Result<String, HandshakeError> {


    // Either accept or open the handshake channel, depending on if this is the server or client
    let channel = if is_server {
        // Accept the first bidirectional stream, ignoring any errors
        connection.accept_bi().await
            .map_err(|e| HandshakeError::ConnectionError(e.into()))?
    } else {
        // The client initializes the stream
        connection.open_bi().await
            .map_err(|e| HandshakeError::ConnectionError(e.into()))?.await
            .map_err(|e| HandshakeError::TransmissionError(e.into()))?
    };

    // Wrap in a framed packet sender
    let mut framed = Framed::<PeerMessage>::new(channel.0, channel.1);

    // If this is the server, anticipate the channel's purpose to be for a handshake.
    if is_server {
        let PeerMessage::Initialize(ChannelPurpose::Handshake) = framed.recv().await? else {
            return Err(HandshakeError::UnexpectedPacket);
        };
    } else {
        // Otherwise, send the channel's purpose as being for a handshake
        framed.send(&PeerMessage::Initialize(ChannelPurpose::Handshake)).await?;
    }

    // Construct the packet containing this peer's info.
    let info_packet = PeerMessage::Handshake { name: peer.name.clone() };

    // Send the peer info packet
    framed.send(&info_packet).await?;

    // Recieve the peer's info packet
    let PeerMessage::Handshake { name } = framed.recv().await? else {
        return Err(HandshakeError::UnexpectedPacket);
    };

    // If the peer's name is in the map, tell the peer and error.
    if peer.peers.read().expect("peers lock not poisoned").contains_key(&name) {
        framed.send(&PeerMessage::HandshakeResponse(HandshakeResponse::NameTaken)).await?;
        return Err(HandshakeError::NameTaken);
    }

    // Otherwise, respond that the handshake succeeded
    framed.send(&PeerMessage::HandshakeResponse(HandshakeResponse::Ok)).await?;

    // Receive the peer's response, returning early if the response was unsatisfactory.
    match framed.recv().await? {
        PeerMessage::HandshakeResponse(HandshakeResponse::Ok) => (),
        PeerMessage::HandshakeResponse(r) => return Err(HandshakeError::UnsatisfactoryResponse(r)),
        _ => return Err(HandshakeError::UnexpectedPacket),
    }

    // Add the peer to the map
    peer.peers.write().expect("peers lock not poisoned")
        .insert(name.clone(), Arc::new(connection));

    // Return the peer's name
    Ok(name)
}