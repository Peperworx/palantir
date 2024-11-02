use std::{io, net::SocketAddr};

use thiserror::Error;
use wtransport::error::StreamWriteError;



#[derive(Debug, Error)]
pub enum PalantirError {
    #[error("{0}")]
    ConnectingError(#[from] ConnectingError),
    #[error("{0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("{0}")]
    HandshakeError(#[from] HandshakeError),
    #[error("{0}")]
    FramedError(#[from] FramedError),
    #[error("palantir is not yet initialized")]
    NotInitialized,
    #[error("palantir is already running")]
    AlreadyRunning,
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
    /// The peer sent an invalid magic vlaue
    #[error("invalid magic number received from peer")]
    InvalidMagic,
    /// The peer tried to use a name that already exists
    #[error("peer sent a name that is already in use")]
    NameTaken,
    #[error("{0}")]
    TransmissionError(#[from] TransmissionError),
    #[error("{0}")]
    ConnectionError(#[from] ConnectionError),
}





/// # [`ConnectionError`]
/// There was an error with the overall webtransport connection
#[derive(Debug, Error)]
pub enum ConnectionError {
    /// The peer closed the connection
    #[error("peer closed connection ({reason})")]
    ConnectionClose {
        reason: String,
    },
    /// This current peer closed the connection
    #[error("connection closed locally")]
    LocallyClosed,
    /// The connection was aborted to to an underlying protocol violation
    #[error("connection aborted due to protocol violation ({reason})")]
    LocallyAborted {
        reason: String,
    },
    /// The connection timed out
    #[error("connection timed out")]
    TimedOut,
    /// The connection was closed due to an error raised by the underlying transport
    #[error("transport error ({reason})")]
    TransportError {
        reason: String,
    }
}

impl From<wtransport::error::ConnectionError> for ConnectionError {
    fn from(value: wtransport::error::ConnectionError) -> Self {
        match value {
            wtransport::error::ConnectionError::ConnectionClosed(connection_close) => 
                Self::ConnectionClose { reason: connection_close.to_string() },
            wtransport::error::ConnectionError::ApplicationClosed(application_close) =>
                Self::ConnectionClose { reason: application_close.to_string() },
            wtransport::error::ConnectionError::LocallyClosed =>
                Self::LocallyClosed,
            wtransport::error::ConnectionError::LocalH3Error(h3_error) =>
                Self::LocallyAborted { reason: h3_error.to_string() },
            wtransport::error::ConnectionError::CidsExhausted =>    
                Self::LocallyAborted { reason: "cids exhausted".to_string() },
            wtransport::error::ConnectionError::TimedOut =>
                Self::TimedOut,
            wtransport::error::ConnectionError::QuicProto(quic_proto_error) =>
                Self::TransportError { reason: quic_proto_error.to_string() },
        }
    }
}



/// # [`ConnectingError`]
/// A client had an error in connecting to a server
#[derive(Debug, Error)]
pub enum ConnectingError {
    /// Invalid connection url or ip address
    #[error("peer address was invalid")]
    InvalidPeerAddress(SocketAddr),
    /// There was an internal error with an invalid url used.
    /// If this error occurs, it's a bug in Replix.
    #[error("invalid url")]
    InvalidUrlOrDomain,
    /// There was a connection error experienced
    #[error("{0}")]
    ConnectionError(#[from] ConnectionError),
    /// The connection was rejected by the remote
    #[error("connection rejected")]
    Rejected,
    /// The local client is in a state such that it can't connect
    #[error("local client can't connect")]
    LocalConnectingError,
}

impl From<wtransport::error::ConnectingError> for ConnectingError {
    fn from(value: wtransport::error::ConnectingError) -> Self {
        match value {
            wtransport::error::ConnectingError::InvalidUrl(_) => Self::InvalidUrlOrDomain,
            wtransport::error::ConnectingError::DnsLookup(_) => Self::InvalidUrlOrDomain,
            wtransport::error::ConnectingError::DnsNotFound => Self::InvalidUrlOrDomain,
            wtransport::error::ConnectingError::ConnectionError(connection_error) => Self::ConnectionError(connection_error.into()),
            wtransport::error::ConnectingError::SessionRejected => Self::Rejected,
            wtransport::error::ConnectingError::ReservedHeader(_) => Self::Rejected,
            wtransport::error::ConnectingError::EndpointStopping => Self::LocalConnectingError,
            wtransport::error::ConnectingError::CidsExhausted => Self::LocalConnectingError,
            wtransport::error::ConnectingError::InvalidServerName(_) => Self::InvalidUrlOrDomain,
            wtransport::error::ConnectingError::InvalidRemoteAddress(socket_addr) => Self::InvalidPeerAddress(socket_addr),
        }
    }
}



/// # [`FramedError`]
/// Represents an error that that can occur during the transmission
/// and reception of framed packets.
#[derive(Debug, Error)]
pub enum FramedError {
    /// A received packet had an invalid encoding
    #[error("received framed packet with invalid encoding")]
    InvalidEncoding {
        packet: Vec<u8>,
    },
    /// A packet was unable to be sent because it exceeds a size limit
    #[error("sent packet of size {packet_size} exceeds size limit of {size_limit} ({reason})")]
    ExceedsSizeLimit {
        packet_size: usize,
        size_limit: usize,
        reason: String,
    },
    /// An error occured during the attempted transmission of a packet
    #[error("{0}")]
    TransmissionError(#[from] TransmissionError)
}

/// # [`TransmissionError`]
/// Represents a general error that could occur during the transmission of a payload.
#[derive(Debug, Error)]
pub enum TransmissionError {
    /// Attempted to send or receive data without being connected
    #[error("attempted to send or receive data without being connected")]
    NotConnected,
    /// A peer disconnected with the given code
    #[error("peer disconnected with code {0:x}")]
    PeerDisconnected(u64),
    /// There was an internal error with the underlying transport.
    /// The reason behind this error is not generally revealed to Replix.
    #[error("error occured in underlying transport")]
    TransportError,
    /// We were expecting to receive a specific number of bytes,
    /// but less were sent than expected.
    #[error("expected to receive {expected_size}, but only received {received_size}")]
    TransmissionEndedEarly {
        expected_size: usize,
        received_size: usize,
        reason: String,
    },
    /// The peer refused to open a new connection
    #[error("peer refused to open a new connection")]
    Refused,
}

impl From<wtransport::error::StreamReadError> for TransmissionError {
    fn from(value: wtransport::error::StreamReadError) -> Self {
        match value {
            wtransport::error::StreamReadError::NotConnected => Self::NotConnected,
            wtransport::error::StreamReadError::Reset(var_int) => Self::PeerDisconnected(var_int.into_inner()),
            wtransport::error::StreamReadError::QuicProto => Self::TransportError,
        }
    }
}


impl From<StreamWriteError> for TransmissionError {
    fn from(value: StreamWriteError) -> Self {
        match value {
            StreamWriteError::NotConnected => Self::NotConnected,
            StreamWriteError::Stopped(var_int) => Self::PeerDisconnected(var_int.into_inner()),
            StreamWriteError::QuicProto => Self::TransportError,
            StreamWriteError::Closed => Self::NotConnected,
        }
    }
}

impl From<wtransport::error::StreamOpeningError> for TransmissionError {
    fn from(value: wtransport::error::StreamOpeningError) -> Self {
        match value {
            wtransport::error::StreamOpeningError::NotConnected => Self::NotConnected,
            wtransport::error::StreamOpeningError::Refused => Self::Refused,
        }
    }
}

