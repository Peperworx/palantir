//! # Frame
//! Provides simple bincode-based packet framing over [`SendStream`]s and [`RecvStream`]s.


use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use wtransport::{error::StreamReadExactError, RecvStream, SendStream, VarInt};

use crate::error::{FramedError, TransmissionError};


/// # [`Framed`]
/// A pair of send and receive channels that are framed
pub struct Framed<P>(pub SendFramed<P>, pub RecvFramed<P>);

impl<P: Serialize + for<'a> Deserialize<'a>> Framed<P> {
    /// # [`Framed::new`]
    /// Create a new pair of framed channels
    pub fn new(send: SendStream, recv: RecvStream) -> Self {
        Self(
            SendFramed::new(send),
            RecvFramed::new(recv)
        )
    }

    /// # [`Framed::send`]
    /// Sends the given packet over the stream
    /// 
    /// # Errors
    /// Returns an error if either the packer failed to serialize, or the data transfer failed.
    pub async fn send(&mut self, packet: &P) -> Result<(), FramedError> {
        self.0.send(packet).await
    }

    /// # [`Framed::recv`]
    /// Receives the next frame of data
    /// 
    /// # Errors
    /// Returns an error if either deserialization fails or the connection is closed
    /// before we can finish receiving.
    pub async fn recv(&mut self) -> Result<P, FramedError> {
        self.1.recv().await
    }
}


impl<P: Serialize + for<'a> Deserialize<'a>> From<(SendStream, RecvStream)> for Framed<P> {
    fn from(value: (SendStream, RecvStream)) -> Self {
        Self::new(value.0, value.1)
    }
}
/// # [`SendFramed`]
/// Send framed packets over a wrapped send stream.
pub struct SendFramed<P>(SendStream, PhantomData<P>);


impl<P: Serialize> SendFramed<P> {

    /// # [`SendFramed::new`]
    /// Creates a new [`SendFramed`] instance wrapping the given [`SendStream`]
    #[must_use]
    pub fn new(stream: SendStream) -> Self {
        Self(stream, PhantomData)
    }

    /// # [`SendFramed::send`]
    /// Sends the given packet over the stream.
    /// 
    /// # Errors
    /// Returns an error if either the packer failed to serialize, or the data transfer failed.
    pub async fn send(&mut self, packet: &P) -> Result<(), FramedError> {

        // Serialize the packet using bincode
        let data: Vec<u8> = postcard::to_allocvec(packet)
            .map_err(|_| FramedError::ExceedsSizeLimit {
                packet_size: std::mem::size_of::<P>(),
                size_limit: usize::MAX, // This should never happen, as the default size config is unlimited
                reason: "bincode was unable to serialize a struct of the given size".to_string(),
            })?;

        // Get the length of the data as little endian bytes.
        // Truncate to a u32, and error if it is too large
        let len = u32::try_from(data.len())
            .map_err(|_| FramedError::ExceedsSizeLimit {
                packet_size: data.len(),
                size_limit: u32::MAX as usize,
                reason: "packets larget than u32::MAX".to_string(),
            })?;
        
        let data_len = len.to_le_bytes();

        // Build a vec of the length followed by the data.
        let mut frame = data_len.to_vec();
        frame.extend(&data); // We already know it is within length.
        
        println!("{:?}, {}", len, frame.len());
        self.0.write_all(&frame).await.map_err(TransmissionError::from)?;

        Ok(())
    }

    /// # [`SendFramed::close`]
    /// Closes the connection with the given code
    pub async fn close(&mut self) {
        self.0.finish().await;
    }
}

/// # [`RecvFramed`]
/// Receive framed packets over a wrapped recv stream.
pub struct RecvFramed<P>(RecvStream, PhantomData<P>);


impl<P: for<'de> Deserialize<'de>> RecvFramed<P> {

    /// # [`RecvFramed::new`]
    /// Creates a new [`RecvFramed`] instance wrapping the given [`RecvStream`]
    #[must_use]
    pub fn new(stream: RecvStream) -> Self {
        Self(stream, PhantomData)
    }

    /// # [`RecvFramed::recv`]
    /// Receives the next frame of data
    /// 
    /// # Errors
    /// Returns an error if either deserialization fails or the connection is closed
    /// before we can finish receiving.
    pub async fn recv(&mut self) -> Result<P, FramedError> {

        // Read in exactly 4 bytes for the u32 length.
        let mut length = [0u8; 4];
        self.0.read_exact(&mut length).await.map_err(|e| match e {
            StreamReadExactError::FinishedEarly(received_size) => TransmissionError::TransmissionEndedEarly {
                expected_size: 4,
                received_size,
                reason: "read frame length".to_string(),
            },
            wtransport::error::StreamReadExactError::Read(stream_read_error) => stream_read_error.into(),
        })?;

        // Convert the u32 from le bytes
        let length = u32::from_le_bytes(length);
        println!("{:?}", length);

        // Allocate exactly `length` bytes to read
        // This cast should be fine. If your usize is too small to hold the size of the message,
        // then your computer will crash anyways because of the allocation.
        // If you do, however, manage to get this all running on a 16 bit computer, please let me know.
        let mut data = vec![0u8; length as usize];

        // Read in said message.
        self.0.read_exact(&mut data).await.map_err(|e| match e {
            StreamReadExactError::FinishedEarly(received_size) => TransmissionError::TransmissionEndedEarly {
                expected_size: 4,
                received_size,
                reason: "read frame data".to_string(),
            },
            wtransport::error::StreamReadExactError::Read(stream_read_error) => stream_read_error.into(),
        })?;

        
        // Deserialize the frame using bincode.
        postcard::from_bytes(&data)
            .map_err(|_| FramedError::InvalidEncoding { packet: data })
    }
}