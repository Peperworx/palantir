//! # Validation
//! This module provides traits and utilities for validating peers that are trying to connect to our local instance.
//! This can be used to implement auth, or simply ensure that clients use the same messages.

use serde::{Deserialize, Serialize};
use wtransport::endpoint::{IncomingSession, SessionRequest};



/// # [`Validator`]
/// A [`Validator`] determines whether or not a client should be permitted
/// to continue connecting at different stages.
pub trait Validator: Send + Sync + 'static {

    /// The data type that a client sends during a handshake with the server
    /// on every new connection.
    type Validation: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static;
    
    /// # [`Validator::validate_incoming_session`]
    /// This method should return `true` if a client sending a given
    /// session should be granted entry, and false otherwise.
    fn validate_incoming_session(&self, incoming: &IncomingSession) -> impl std::future::Future<Output = bool> + Send;

    /// # [`Validator::validate_session_request`]
    /// This method should return `true` if a client sending a given
    /// session request should be granted access, and false otherwise.
    fn validate_session_request(&self, request: &SessionRequest) -> impl std::future::Future<Output = bool> + Send;

    /// # [`Validator::validate_validation`]
    /// This method should return `true` if a client sending a given
    /// validation should be granted access, and false otherwise.
    fn validate_validation(&self, validation: &Self::Validation) -> impl std::future::Future<Output = bool> + Send;
}


impl<
    A: Validator<Validation = V>,
    B: Validator<Validation = V>,
    V: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static
    > Validator for (A, B) {
    type Validation = V;

    async fn validate_incoming_session(&self, incoming: &IncomingSession) -> bool {
        self.0.validate_incoming_session(incoming).await &
        self.1.validate_incoming_session(incoming).await
    }

    async fn validate_session_request(&self, request: &SessionRequest) -> bool {
        self.0.validate_session_request(request).await &
        self.1.validate_session_request(request).await
    }

    async fn validate_validation(&self, validation: &Self::Validation) -> bool {
        self.0.validate_validation(validation).await &
        self.1.validate_validation(validation).await
    }
}