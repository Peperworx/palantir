//! # Validation
//! This module provides traits and utilities for validating peers that are trying to connect to our local instance.
//! This can be used to implement auth, or simply ensure that clients use the same messages.

use serde::{Deserialize, Serialize};
use wtransport::endpoint::{IncomingSession, SessionRequest};

use crate::message::Side;




/// # [`Validator`]
/// A [`Validator`] determines whether or not a client should be permitted
/// to continue connecting at different stages.
pub trait Validator: Send + Sync + 'static {

    /// The state type that is maintained during the connection.
    /// This can be used for anything the validator needs.
    type State: Sized;

    /// The [`Validator`] is provided a window
    /// during the handshake to perform its own operations.
    /// This is the packet type used during this window.
    type Packet: Serialize + for<'a> Deserialize<'a>;
    
    /// # [`Validate::create_new_state`]
    /// Creates a new instance of [`Validator::State`] that will be used
    /// during communications with the given peer.
    async fn create_new_state(&self, mode: Side) -> Self::State;

    
}

