//! # Palantir

#[warn(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]


mod request;
mod actor_id;
use actor_id::ActorID;
use request::Request;




use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};



pub struct Palantir<B> {
    /// This system's id
    system_id: String,
    /// The backend that is used by this palantir instance
    /// to communicate with other systems.
    backend: B,
    /// A hashmap of message handling channels for actors
    actor_handlers: RwLock<HashMap<ActorID, mpsc::Sender<Request>>>,
}