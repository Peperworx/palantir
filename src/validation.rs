//! # Validation
//! This module provides traits and utilities for validating peers that are trying to connect to our local instance.
//! This can be used to implement auth, or simply ensure that clients use the same messages.

use serde::{Deserialize, Serialize};
use wtransport::{endpoint::{IncomingSession, SessionRequest}, Connection};


use crate::{error::ConnectionError, frame::Framed};




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
    type Packet: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static;
    
    /// # [`Validator::create_new_state`]
    /// Creates a new instance of [`Validator::State`] that will be used
    /// during the validation of a server from the client side.
    async fn create_new_state(&self) -> Self::State;

    /// # [`Validator::validate_incoming_session`]
    /// Validates the given incoming session. This method should
    /// return a new [`Self::State`] if the client should be allowed to continue connecting.
    /// This will be called on the server side.
    async fn validate_incoming_session(&self, incoming: &IncomingSession) -> Option<Self::State>;

    /// # [`Validator::validate_session_request`]
    /// Validates the given session request. This should return `true`
    /// if the connection should be allowed to continue, or `false` if
    /// the `forbidden` response should be sent.
    /// This will be called on the server side.
    async fn validate_session_request(&self, session: &SessionRequest, state: &mut Self::State) -> bool;

    /// # [`Validator::validate_connection`]
    /// Validates the given connection. This should return `true`
    /// if operations should continue, or `false` if the connection should be dropped.
    /// An error will also cause the connection to be dropped.
    /// This will be called on both the client and server side, and can be used to implement
    /// things such as handshakes. To determine which end of the connection this validator is on,
    /// it is recomemnded to store a value in the state when it is created, as two different methods
    /// are used to create the state on the client and server side.
    async fn validate_connection(&self, connection: &Connection, state: &mut Self::State) -> Result<bool, ConnectionError>;


}

