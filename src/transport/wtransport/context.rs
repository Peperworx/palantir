//! # Context
//! Internal structs used to store sharable data used by tasks spawned by a [`DirectPeer`]s
//! main loop.

use std::sync::Arc;

use super::DirectPeer;

#[derive(Clone)]
pub(super) struct DirectPeerContext {
    /// An Arc of the [`DirectPeer`] this context is being used for
    pub(super) direct_peer: Arc<DirectPeer>,
    /// A clone of the send channel used for communication back to the main loop
    pub(super) sender: kanal::AsyncSender<()>,
    
}