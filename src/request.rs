//! # Request
//! Provides the [`Request`] struct, which is a basic structure for providing request/response semantics over mpsc channels.



use tokio::sync::oneshot;

/// # [`Request`]
/// Basic struct that provides request/response semantics over mpsc channels
pub struct Request {
    /// The request's data
    pub(crate) data: Vec<u8>,
    /// The request's responder
    pub(crate) responder: oneshot::Sender<Vec<u8>>
}

impl Request {
    /// # [`Request::new`]
    /// Creates a new [`Request`] instance with the given data,
    /// returning the [`Request`] and the response [`oneshot`]
    pub fn new(data: Vec<u8>) -> (Self, oneshot::Receiver<Vec<u8>>) {

        let (responder, response) = oneshot::channel();

        (Self {
            data,
            responder,
        }, response)
    }

    /// # [`Request::data`]
    /// Returns the request's data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// # [`Request::respond`]
    /// Responds to the request, consuming this request object.
    /// 
    /// # Errors
    /// If the response fails, this returns the response data as an error.
    pub fn respond(self, response: Vec<u8>) -> Result<(), Vec<u8>> {
        self.responder.send(response)
    }
}