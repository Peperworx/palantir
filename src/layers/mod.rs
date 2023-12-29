//! # Layers

use core::future::Future;
use std::error::Error;


pub mod web_transport;

/// # [`Layer`]
/// A [`Layer`] allows sending a message `M` to a peer and waits for a response `R`.
pub trait Layer<M, R>: Send {
    /// The error returned by the layer
    type Error: Error;

    /// The peer ID type
    type PeerID;

    /// Send a message to the given peer and wait for a response
    fn request(&self, peer: Self::PeerID, message: M) -> impl Future<Output = Result<R, Self::Error>>;
}