//! # Channels
//! Channels provide a simple interface to request/response semantics implemented
//! on top of a WebTransport channel.

use std::{future::Future, sync::Arc, time::Duration};


use crate::{error::FramedError, frame::{RecvFramed, SendFramed}, timeout::TimeoutChannels};

use super::{message::{PeerMessage, RequestID}, Peer};





pub struct Channel {
    /// The framed sender. This will be locked by every request.
    sender: SendFramed<PeerMessage>,
    /// This is essentially a slotmap containing oneshot channels that are used to send responses
    /// to a request, but it also implements some more complex timeout logic.
    responders: Arc<TimeoutChannels<RequestID, Vec<u8>>>,
}


impl Channel {
    /// # [`Channel::new`]
    /// Creates a new channel wrapping a framed sender
    #[must_use]
    pub fn new(sender: SendFramed<PeerMessage>) -> Self {
        Self {
            sender,
            responders: TimeoutChannels::new(Duration::from_secs(30)).into(),
        }
    }

    /// # [`Channel::request`]
    /// Send a request, and wait for the response
    /// 
    /// # Errors
    /// Errors if a framed send fails. Will also error with a transport error if the timeout is reached while waiting for a response.
    pub async fn request(&mut self, data: Vec<u8>) -> Result<Vec<u8>, FramedError> {


        // Create the responder
        let (response, id) = self.responders.add();

        // Construct the request
        let request = PeerMessage::Request {
            request_id: id,
            body: data
        };

        // Send the request
        self.sender.send(&request).await?;

        // Wait for a response
        response.await.map_err(|_| FramedError::TransmissionError(crate::error::TransmissionError::TransportError))
    }

    /// # [`Channel::create_run_future`]
    /// Creates the run future for this channel from self and the framed receiver,
    /// returning it. This eliminates the need for every member of channel to be allocated
    /// in an Arc, and also eliminates some of the need for interior mutability and locks.
    pub(crate) fn create_run_future(&self, recv: RecvFramed<PeerMessage>) -> impl Future<Output = ()> {
        Self::run(recv, self.responders.clone())
    }

    /// # [`Channel::run`]
    /// Channels need a running main loop to dispatched recieved data.
    /// This method should be run in a separate task.
    async fn run(mut recv: RecvFramed<PeerMessage>, responders: Arc<TimeoutChannels<RequestID, Vec<u8>>>) {

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
            let Some(responder) = responders.pop(request_id) else {
                // If the responder has timed out, just ignore it
                continue;
            };

            // Send the response. Ignore any errors
            let _ = responder.send(body);

        }
    }
}