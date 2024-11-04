//! # Palantir

#[warn(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]


mod request;
mod actor_id;
use actor_id::ActorID;
use fluxion::{Delegate, Handler, IndeterminateMessage, LocalRef};
use request::Request;
use serde::{Deserialize, Serialize};




use std::{collections::HashMap, sync::Arc};
use tokio::{sync::{mpsc, RwLock}, task::JoinSet};


/// # [`Palantir`]
/// Palantir provides a [`Delegate`] implementation for [`fluxion`] that is generic over [`Backends`].
/// Generally, this is used to connect a [`fluxion`] system to a network.
pub struct Palantir<B> {
    /// This system's id
    system_id: String,
    /// The backend that is used by this palantir instance
    /// to communicate with other systems.
    backend: B,
    /// A hashmap of message handling channels for actors
    actor_handlers: RwLock<HashMap<(ActorID, String), mpsc::Sender<Request>>>,
    /// A join set containing tasks spawned by this palantir instance
    join_set: Arc<std::sync::Mutex<JoinSet<()>>>,
}


impl<B> Palantir<B> {
    /// # [`Palantir::new`]
    /// Creates a new [`Palantir`] instance with the given system id and backend.
    pub fn new(system_id: String, backend: B) -> Self {
        Self {
            system_id,
            backend,
            actor_handlers: Arc::default(),
        }
    }
}

impl<B> Palantir<B> {
    /// # [`Palantir::register`]
    /// Registers a specific actor as being capable of communicating over the backend with a specific message type.
    pub fn register<A: Handler<M>, M: IndeterminateMessage, D: Delegate + AsRef<Self>>(&self, actor: LocalRef<A, D>)
        where M::Result: Serialize + for<'de> Deserialize<'de> {

        // TODO: Remove this and replace with proper logging
        println!("{} is registering actor with id {} to handle message {}", self.system_id, actor.get_id(), M::ID);

        // Create the request channels
        let (request_sender, request_receiver) = mpsc::channel::<Request>(256);

        // Clone off the join set for the spawned task
        let join_set_clone = self.join_set.clone();
        
        // Lock the join set
        let join_set = self.join_set.lock().expect("join set mutex should never be poisoned");

        // Spawn a task that deserializes and relays messages to the actor
        join_set.spawn(async move {
            // The main loop for receiving this type of message for this specific actor
            loop {

                // Receive the next message.
                let Some(next_message) = request_receiver.recv().await else {
                    // TODO: Better logging.
                    // This point will only ever be reached if there are no longer
                    // any senders, which means there will never be any others.
                    // While this should be logged, it doesn't necessarily
                    // mean that the palantir instance is broken, just that
                    // this type of message will never be received again.
                    println!("Message handler {}/{} stopped recieving messages.", actor.get_id() ,M::ID);
                    break;
                };

                // Clone the actor ref
                let actor = actor.clone();

                // Spawn a new task handling the message
                join_set_clone.lock().expect("join set mutex should never be poisoned")
                    .spawn(async move {
                        // Deserialize the message.
                        // While the deserialization shouldn't fail, as the message types should be known ahead of time,
                        // there does exist a possibility that two peers have different versions of the message.
                        // As palantir doesn't yet support message schema validation (it may in the future,
                        // and this is actually what the introspectable crate was initially created for),
                        // we will simply ignore messages that don't deserialize properly.
                        let Ok(message) = postcard::from_bytes::<M>(next_message.data()) else {
                            return;
                        };

                        // 
                    });

            }
        });

        // Drop the join set guard so we don't hold it over the actor handlers lock's await point.
        drop(join_set);
        
        todo!()
    }
}