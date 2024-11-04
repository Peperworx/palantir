use std::io;

use thiserror::Error;

use crate::error::{ConnectionError, FramedError, TransmissionError};

use super::message::HandshakeResponse;



#[derive(Error, Debug)]
pub enum AddPeerError {}

#[derive(Error, Debug)]
pub enum OpenChannelError {
    /// This error is returned when a peer does not have an active connection with another peer.
    #[error("peer does not exist")]
    PeerDoesntExist,
    /// A connection error was returned when working directly with the wtransport channel.
    #[error("{0}")]
    ConnectionError(#[from] ConnectionError),
    /// A transmission error occurred while working directly with the wtransport channel.
    #[error("{0}")]
    TransmissionError(#[from] TransmissionError),
    /// There was an error conducting a framed send
    #[error("{0}")]
    FramedError(#[from] FramedError)
}

#[derive(Error, Debug)]
pub enum RunPeerError {
    #[error("experienced error while creating endpoint")]
    EndpointCreationError {
        #[source]
        source: io::Error,
    }
}


/// # [`HandshakeError`]
/// Error that occurs during a handshake with a peer
#[derive(Debug, Error)]
pub enum HandshakeError {
    
    /// There was a framed error during the handshake
    #[error("{0}")]
    FramedError(#[from] FramedError),
    /// The peer sent an unexpected packet
    #[error("unexpected packet recieved from peer")]
    UnexpectedPacket,
    /// The peer tried to use a name that already exists
    #[error("peer sent a name that is already in use")]
    NameTaken,
    /// The peer responded with an unsatisfactory response
    #[error("unsatisfactory response from peer")]
    UnsatisfactoryResponse(HandshakeResponse),
    #[error("{0}")]
    TransmissionError(#[from] TransmissionError),
    #[error("{0}")]
    ConnectionError(#[from] ConnectionError),
}