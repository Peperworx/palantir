//! # Channels
//! Channels provide a simple interface to request/response semantics implemented
//! on top of a WebTransport channel.

use std::time::Duration;

use tokio::sync::Mutex;

use crate::{error::FramedError, frame::{Framed, RecvFramed, SendFramed}, timeout::TimeoutChannels};

use super::{message::{PeerMessage, RequestID}, Peer};




pub struct Channel {
    /// The framed receiver. Even though this is behind a mutex,
    /// it will only ever be locked in a single place, in the run method.
    receiver: Mutex<RecvFramed<PeerMessage>>,
    /// The framed sender. This will be locked by every request.
    sender: Mutex<SendFramed<PeerMessage>>,
    /// This is essentially a slotmap containing oneshot channels that are used to send responses
    /// to a request, but it also implements some more complex timeout logic.
    responders: TimeoutChannels<RequestID, Vec<u8>>,
}


impl Channel {
    /// # [`Channel::new`]
    /// Creates a new channel wrapping a framed sender and receiver
    #[must_use]
    pub fn new(framed: Framed<PeerMessage>) -> Self {
        Self {
            receiver: Mutex::new(framed.1),
            sender: Mutex::new(framed.0),
            responders: TimeoutChannels::new(Duration::from_secs(30)),
        }
    }

    /// # [`Channel::request`]
    /// Send a request, and wait for the response
    /// 
    /// # Errors
    /// Errors if a framed send fails. Will also error with a transport error if the timeout is reached while waiting for a response.
    pub async fn request(&self, data: Vec<u8>) -> Result<Vec<u8>, FramedError> {

        // Lock the sender
        let mut sender = self.sender.lock().await;

        // Create the responder
        let (response, id) = self.responders.add();

        // Construct the request
        let request = PeerMessage::Request {
            request_id: id,
            body: data
        };

        // Send the request
        sender.send(&request).await?;

        // Hold the guard while waiting for a response
        drop(sender);

        // Wait for a response
        response.await.map_err(|_| FramedError::TransmissionError(crate::error::TransmissionError::TransportError))
    }

    /// # [`Channel::run`]
    /// Channels need a running main loop to dispatched recieved data.
    /// This method should be run in a separate task.
    pub async fn run(&self) {
        // Lock the reciever mutex
        let mut recv = self.receiver.lock().await;

        // If we get 5 errors in a row, we need to quit
        let mut error_counter = 0;

        // Recieve packets in a loop
        while error_counter < 5 {

            // Recieve the next packet
            let Ok(next) = recv.recv().await else {
                // Ignore errors for now, but once we
                // reach five errors, we need to quit the loop
                error_counter += 1;
                continue;
            };

            // Reset error counter back to zero
            error_counter = 0;

            // Destructure the message
            let PeerMessage::Response { request_id, body } = next else {
                // Ignore incorrect packets
                continue;
            };

            // Retrieve the response sender for the ID
            let Some(responder) = self.responders.pop(request_id) else {
                // If the responder has timed out, just ignore it
                continue;
            };

            // Send the response. Ignore any errors
            let _ = responder.send(body);

        }
    }
}