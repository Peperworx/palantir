//! # Web Transport Layers
//! This module contains ['Layer'] implementations that utilize webtransport as a backend.
//! There are two of these layers: A host layer and a client layer.
//! The host layer contains a webtransport server that can communicate with every client connected to it.
//! The client layer contains a webtransport client that can only communicate with the host.
//! Peers are identified using the [`HostedPeerID`] enum, which is provided under the [`crate::identification`] module.

use thiserror::Error;
use serde::{Serialize, Deserialize};
use wtransport::{error::{ConnectingError, ConnectionError, StreamOpeningError, StreamReadExactError, StreamReadError, StreamWriteError}, RecvStream, SendStream};

use crate::identification::HostedPeerID;



pub mod namespace;
pub mod peer;

pub mod client;



/// # [`WebTransportLayerError`]
/// Enum containing every error that can occur in either webtransport layer.
#[derive(Error, Debug)]
pub enum WebTransportLayerError {
    /// Error returned when a given peer does not exist
    #[error("peer '{0:?}' does not exist")]
    PeerDoesNotExist(HostedPeerID),
    /// Error returned when the webtransport client fails to connect
    #[error("connecting errorr {0}")]
    ConnectingError(#[from] ConnectingError),
    /// Error returned during a connection. Not to be confused with ConnectingError
    #[error("connection error {0}")]
    ConnectionError(#[from] ConnectionError),
    /// Error returned while openning a stream
    #[error["stream openning error {0}"]]
    StreamOpeningError(#[from] StreamOpeningError),
    /// Error serializing or deserializing data
    #[error("serialization error {0}")]
    SerializationError(#[from] bincode::Error),
    /// Error reading an exact quantity from a stream
    #[error("read exact error {0}")]
    ReadExactError(#[from] StreamReadExactError),
    /// Error reading from the stream
    #[error("stream read error {0}")]
    StreamReadError(#[from] StreamReadError),
    /// Error writing to stream
    #[error("stream write error {0}")]
    StreamWriteError(#[from] StreamWriteError),
    /// An IO Error occured
    #[error("IO Error {0}")]
    IOError(#[from] std::io::Error),
    /// An error occured while opening a namespace
    #[error("Error opening namespace")]
    NamespaceOpenError,
    /// An error occured while reading from a namespace. Invalid packet received
    #[error("Invalid namespace packet")]
    InvalidNSPacket
}


/// # [`WTNamespaceID`]
/// The namespace IDs used by the WebTransport layer internally
#[derive(Serialize, Deserialize, PartialEq, Clone, Eq)]
pub enum WTNamespaceID {
    /// The core namespace, used for direct communication
    Core,
    /// A named namespace
    Named(String),
    /// A named namespace on another peer
    Peer(HostedPeerID, String),
}

/// # [`WebTransportPacket`]
/// The packet type used by the webtransport layer when initializing a namespace
#[derive(Serialize, Deserialize, PartialEq)]
enum WebTransportPacket {
    /// Sent by the client, triggers the initialization of a namespace.
    InitializeNamespace(WTNamespaceID),
    /// Response to [`InitializeNamespace`]. If true, the connection successfully initialized the namespace.
    /// If false, the namespace does not exist.
    /// Future versions of this response may change. 
    NamespaceInitResponse(bool),
    /// Send raw bytes along the namespace
    Bytes(Vec<u8>),
}




/// # [`WebTransportCodec`]
/// Wraps a bidirectional WebTransport stream, enabling [`WebTransportPacket`]s to be read and written over the stream.
struct WebTransportCodec {
    /// The raw send stream
    send: SendStream,
    /// The raw receive stream
    recv: RecvStream,
    /// The receive buffer
    buf: Vec<u8>,
}

impl WebTransportCodec {
    /// Create a new [`WebTransportCodec`]
    pub fn new(send: SendStream, recv: RecvStream) -> Self {
        Self {
            send, recv,
            buf: Vec::new()
        }
    }

    /// Send a packet
    pub async fn send(&mut self, packet: &WebTransportPacket) -> Result<(), WebTransportLayerError> {

        // Encode the packet as bytes
        let encoded = bincode::serialize(packet)?;

        // Get the packet length and encode it as little endian.
        let mut packet = encoded.len().to_le_bytes().to_vec();

        // Add the encoded bytes
        packet.extend(encoded);

        // Send the packet
        self.send.write_all(&packet).await?;

        Ok(())
    }

    /// Receive a packet
    pub async fn recv(&mut self) -> Result<WebTransportPacket, WebTransportLayerError> {

        // Read in a usize
        let mut length = [0u8; std::mem::size_of::<usize>()];
        self.recv.read_exact(&mut length).await?;
        
        // Convert from LE bytes
        let length = usize::from_le_bytes(length);

        // Read that many more bytes
        loop {
            // If we have enough bytes, then we can break
            if self.buf.len() >= length {
                break;
            }

            // Read more bytes
            let mut buf = [0u8; 1024];
            let len_read = self.recv.read(&mut buf).await?;

            // Append to buffer
            if let Some(len_read) = len_read {
                self.buf.extend(&buf[..len_read]);
            }
        }

        // Cut off the read bytes from the buffer
        let packet = self.buf[..length].to_vec();
        self.buf = self.buf[length..].to_vec();

        // Deserialize the packet
        let packet: WebTransportPacket = bincode::deserialize(&packet)?;

        Ok(packet)
    }
}

