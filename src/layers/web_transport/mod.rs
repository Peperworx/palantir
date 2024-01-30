//! # Web Transport Layers
//! This module contains ['Layer'] implementations that utilize webtransport as a backend.
//! There are two of these layers: A host layer and a client layer.
//! The host layer contains a webtransport server that can communicate with every client connected to it.
//! The client layer contains a webtransport client that can only communicate with the host.
//! Peers are identified using the [`HostedPeerID`] enum, which is provided under the [`crate::identification`] module.

use thiserror::Error;
use serde::{Serialize, Deserialize};
use wtransport::error::{ConnectingError, ConnectionError, StreamOpeningError, StreamReadExactError, StreamReadError, StreamWriteError};

use crate::identification::HostedPeerID;



pub mod namespace;
pub mod peer;

pub mod client;



/// # [`WTLayerError`]
/// Enum containing every error that can occur in either webtransport layer.
#[derive(Error, Debug)]
pub enum WTLayerError {
    /// Error in a stream operation
    #[error("WebTransport Stream Error {0:?}")]
    WTStreamError(#[from] WTStreamError),
    /// Error in a connection
    #[error("WebTransport Connection Error {0:?}")]
    WTConnectionError(#[from] WTConnectionError),

    /// Error serializing or deserializing data
    #[error("serialization error {0}")]
    SerializationError(#[from] bincode::Error),

    /// An IO Error occured
    #[error("IO Error {0}")]
    IOError(#[from] std::io::Error),

    /// Attempted to open a namespace that already exists
    #[error("attempting to open a namespace that already exists: `{0:?}`")]
    NamespaceExists(WTNamespaceID),
    /// Namespace opening denied
    #[error("namespace opening denied by peer")]
    NamespaceDenied,
    /// An error occured while reading from a namespace. Invalid packet received
    #[error("Invalid namespace packet")]
    InvalidNSPacket,
}

#[derive(Error, Debug)]
pub enum WTStreamError {
    /// Error writing to stream
    #[error("stream write error {0}")]
    WriteError(#[from] StreamWriteError),
    /// Error reading from the stream
    #[error("stream read error {0}")]
    ReadError(#[from] StreamReadError),
    /// Error reading an exact quantity from a stream
    #[error("read exact error {0}")]
    ReadExactError(#[from] StreamReadExactError),
    /// Error returned while openning a stream
    #[error["stream openning error {0}"]]
    OpeningError(#[from] StreamOpeningError),

}

#[derive(Error, Debug)]
pub enum WTConnectionError {
    /// Error returned during a connection. Not to be confused with ConnectingError
    #[error("connection error {0}")]
    ConnectionError(#[from] ConnectionError),

    /// Error returned when the webtransport client fails to connect
    #[error("connecting errorr {0}")]
    ConnectingError(#[from] ConnectingError),
}

/*
/// Error returned when a given peer does not exist
    #[error("peer '{0:?}' does not exist")]
    PeerDoesNotExist(HostedPeerID),
    

    
    
    
    

     */


/// # [`WTNamespaceID`]
/// The namespace IDs used by the WebTransport layer internally
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub enum WTNamespaceID {
    /// The core namespace, used for direct communication
    Core,
    /// A named namespace
    Named(String),
    /// A named namespace on another peer
    Peer(HostedPeerID, String),
}

/// # [`WTNSInitPacket`]
/// The packet type used by the webtransport layer when initializing a namespace
#[derive(Serialize, Deserialize, PartialEq)]
enum WTNSInitPacket {
    /// Sent by the client, triggers the initialization of a namespace.
    InitializeNamespace(WTNamespaceID),
    /// Response to [`InitializeNamespace`]. If true, the connection successfully initialized the namespace.
    /// If false, the namespace does not exist.
    /// Future versions of this response may change. 
    NamespaceInitResponse(bool),
}


