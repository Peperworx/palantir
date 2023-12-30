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

    /// The namespace ID type
    type NamespaceID;

    /// Send a message to the given namespace on a peer, and wait for a response
    fn request(&self, namespace: Self::NamespaceID, peer: Self::PeerID, message: M) -> impl Future<Output = Result<R, Self::Error>>;
}