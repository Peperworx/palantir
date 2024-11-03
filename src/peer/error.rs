use thiserror::Error;

use crate::error::{ConnectionError, FramedError, TransmissionError};



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

