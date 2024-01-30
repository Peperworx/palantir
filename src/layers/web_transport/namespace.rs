//! # Webtransport Namespace
//! [`WTNamespace`] wraps a webtransport stream pair and provides a [`Namespace`] implementation fo rthem

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use wtransport::{RecvStream, SendStream};

use crate::layers::Namespace;

use super::{WTLayerError, WTNamespaceID, WTStreamError};



/// # [`WTNamespace`]
/// Wraps a webtransport stream pair and provides a [`Namespace`] implementation.
pub struct WTNamespace<P>{
    /// The stream pair
    stream: (SendStream, RecvStream),
    /// The receive buffer
    buf: Vec<u8>,
    /// The namespace's ID
    id: Option<WTNamespaceID>,
    /// Phantom data
    _phantom: PhantomData<P>
}

impl<P> WTNamespace<P> {
    /// Create a new [`WTNamespace`]
    pub fn new(stream: (SendStream, RecvStream), id: Option<WTNamespaceID>) -> Self {
        Self {
            stream,
            id,
            buf: Vec::new(),
            _phantom: Default::default(),
        }
    }

    /// Send raw bytes
    pub(crate) async fn send_raw(&mut self, bytes: Vec<u8>) -> Result<(), WTStreamError> {


        // Get the packet length and encode it as little endian.
        let mut packet = bytes.len().to_le_bytes().to_vec();

        // Add the encoded bytes
        packet.extend(bytes);

        // Send the packet
        self.stream.0.write_all(&packet).await?;

        Ok(())
    }

    /// Receive raw bytes
    pub(crate) async fn recv_raw(&mut self) -> Result<Vec<u8>, WTStreamError> {

        // Read in a usize
        let mut length = [0u8; std::mem::size_of::<usize>()];
        self.stream.1.read_exact(&mut length).await?;
        
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
            let len_read = self.stream.1.read(&mut buf).await?;

            // Append to buffer
            if let Some(len_read) = len_read {
                self.buf.extend(&buf[..len_read]);
            }
        }

        // Cut off the read bytes from the buffer
        let packet = self.buf[..length].to_vec();
        self.buf = self.buf[length..].to_vec();

        Ok(packet)
    }

    // Set the ID
    pub(crate) fn set_id(&mut self, id: WTNamespaceID) {
        self.id = Some(id);
    }
}


impl<P: Serialize + for<'a> Deserialize<'a>> Namespace for WTNamespace<P> {
    type ID = WTNamespaceID;

    type Packet = P;

    type Error = WTLayerError;

    async fn send(&mut self, packet: &Self::Packet) -> Result<(), Self::Error> {
        
        // Serialize the packet using bincode
        let encoded = bincode::serialize(packet)?;

        // Send the raw bytes
        self.send_raw(encoded).await?;

        Ok(())
    }

    async fn recv(&mut self) -> Result<Self::Packet, Self::Error> {

        // Receive raw bytes
        let encoded = self.recv_raw().await?;

        // Decode using bincode
        let packet = bincode::deserialize(&encoded)?;

        // Return the decoded packet
        Ok(packet)
    }

    fn get_id(&self) -> Option<Self::ID> {
        self.id.clone()
    }
}
