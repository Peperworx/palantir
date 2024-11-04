//! # Palantir

#[warn(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]


pub mod backend;

mod request;
mod actor_id;
use actor_id::ActorID;
use backend::Backend;
use fluxion::{Delegate, Handler, Identifier, IndeterminateMessage, LocalRef, MessageSender};
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
    actor_handlers: RwLock<HashMap<(u64, String), mpsc::Sender<Request>>>,
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
            actor_handlers: RwLock::default(),
            join_set: Arc::default(),
        }
    }
}

impl<B> Palantir<B> {
    /// # [`Palantir::register`]
    /// Registers a specific actor as being capable of communicating over the backend with a specific message type.
    pub async fn register<A: Handler<M>, M: IndeterminateMessage, D: Delegate + AsRef<Self>>(&self, actor: LocalRef<A, D>)
        where M::Result: Serialize + for<'de> Deserialize<'de> {

        // Get the actor's ID, as we will need to hold it after
        // we move the actor to a separate task
        let id = actor.get_id();

        // TODO: Remove this and replace with proper logging
        println!("{} is registering actor with id {} to handle message {}", self.system_id, actor.get_id(), M::ID);

        // Create the request channels
        let (request_sender, mut request_receiver) = mpsc::channel::<Request>(256);

        // Clone off the join set for the spawned task
        let join_set_clone = self.join_set.clone();
        
        // Lock the join set
        let mut join_set = self.join_set.lock().expect("join set mutex should never be poisoned");

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

                        // Handle the message
                        let res = actor.send(message).await;

                        // Serialize it. There shouldn't be any issue serializing the response, but if it doesn't
                        // work there is not much we can do about it
                        let Ok(response) = postcard::to_allocvec(&res) else {
                            return;
                        };

                        // Send the response. Again, nothing we can really do about an error here
                        let _ = next_message.respond(response);
                    });

            }
        });

        // Drop the join set guard so we don't hold it over the actor handlers lock's await point.
        drop(join_set);

        // Add the handler to the map.
        self.actor_handlers.write().await
            .insert((id, M::ID.to_string()), request_sender);
        
    }
}

impl<B: Backend> Delegate for Palantir<B> {
    async fn get_actor<A: Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>> 
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {
        
        // We can't route to actors that are on this peer, so we will return [`None`] if the foreign system id is not provided.
        let (system, id) = match id {
            Identifier::Foreign(id, system) => Some((system, ActorID::Numeric(id))),
            Identifier::ForeignNamed(name, system) => Some((system, ActorID::Named(name.to_string()))),
            _ => None,
        }?;

        // Retrieve a channel to the actor
        let channel = self.backend.open_channel::<M>(id, system).await?;

        // Wrap the 
        todo!()
    }
}

/// # [`PalantirSender`]
/// Implements [`MessageSender`] for communication with [`Palantir`].
/// This is not exposed to the public API directly, and is only ever
/// exposed indirectly via a dyn [`MessageSender`].
struct PalantirSender<M> {
    /// The channel 
}