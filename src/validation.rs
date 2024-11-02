//! # Validation
//! This module provides traits and utilities for validating peers that are trying to connect to our local instance.
//! This can be used to implement auth, or simply ensure that clients use the same messages.

use serde::{Deserialize, Serialize};


use crate::{error::HandshakeError, frame::Framed, message::{PalantirMessage, Side}};




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
    
    /// # [`Validator::create_new_state`]
    /// Creates a new instance of [`Validator::State`] that will be used
    /// during communications with the given peer.
    async fn create_new_state(&self, mode: Side) -> Self::State;

    /// # [`Validator::handshake`]
    /// Runs the validator's handshake
    async fn handshake(&self, framed: &mut Framed<PalantirMessage<Self>>, state: &mut Self::State, name: &str) -> Result<(), Vec<HandshakeError>> where Self: std::marker::Sized;
}

