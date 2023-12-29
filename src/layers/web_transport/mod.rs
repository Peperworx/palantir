//! # Web Transport Layers
//! This module contains ['Layer'] implementations that utilize webtransport as a backend.
//! There are two of these layers: A host layer and a client layer.
//! The host layer contains a webtransport server that can communicate with every client connected to it.
//! The client layer contains a webtransport client that can only communicate with the host.
//! Peers are identified using the [`HostedPeerID`] enum, which is provided under the [`crate::identification`] module.

use thiserror::Error;

use crate::identification::HostedPeerID;


pub mod client;


/// # [`WebTransportLayerError`]
/// Enum containing every error that can occur in either webtransport layer.
#[derive(Error, Debug)]
pub enum WebTransportLayerError {
    /// Error returned when a given peer does not exist
    #[error("peer '{0:?}' does not exist")]
    PeerDoesNotExist(HostedPeerID),
}