//! # Backend
//! [`Backend`]s provide palantir instances connectivity to other instances.



use fluxion::{IndeterminateMessage, Message, MessageSendError};
use serde::{Deserialize, Serialize};

use crate::actor_id::ActorID;



/// # [`Backend`]
/// Provides a palantir instance connectivity to other palantir instances.
pub trait Backend: Send + Sync + 'static {
    /// # [`Backend::Channel`]
    /// The type that implements [`Channel`] for this backend.
    type Channel: Channel;

    /// # [`Backend::open_channel`]
    /// Opens a channel with the given message type, to the given actor, on the given system.
    /// Returns [`None`] if either the system can not be reached, the actor does not exist,
    /// or the actor does not communicate using the given message type.
    fn open_channel<M: Message>(&self, actor: ActorID, system: &str, message_type: &'static str) -> impl std::future::Future<Output = Option<Self::Channel>> + Send;
}

/// # [`Channel`]
/// [`Channel`] implementors represent a single unit of request/response communication
/// of a specific message type, with a specific actor, on a specific system.
pub trait Channel: Send + Sync + 'static {

    /// # [`Channel::request`]
    /// Sends data to the actor, and waits for a response.
    /// This method should return a [`MessageSendError`] in case of an error in transmission.
    fn request(&self, data: Vec<u8>) -> impl std::future::Future<Output = Result<Vec<u8>, MessageSendError>> + Send;


}