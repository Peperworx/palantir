//! # Palantir

#[warn(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]

pub mod error;
pub mod validation;

use std::{collections::HashMap, marker::PhantomData, net::IpAddr, sync::Arc};

use error::PalantirError;
use fluxion::{Delegate, Fluxion, Identifier, IndeterminateMessage, MessageSender};
use tokio::{sync::Mutex, task::JoinSet};
use validation::Validator;
use wtransport::{Connection, Endpoint, Identity, ServerConfig};


/// # [`Palantir`]
/// Palantir enables rudimentary networking for [`fluxion`] via [`wtransport`].
pub struct Palantir<V> {
    /// The port that Palantir will listen on when [`Palantir::run`]
    /// is called. This will be a UDP port, as Palantir uses WebTransport.
    listen_port: u16,
    /// This hashmap contains a list of this instance's connection to peers.
    /// Because this will need to be accessed concurrently, a mutex is used.
    /// The tokio mutex is used because it will need to be held across await
    /// points to send data to a client.
    peers: Mutex<HashMap<String, ()>>,
    /// A joinset containing all tasks spawned by this palantir instance.
    /// This is created to ensure that every task created by palantir is
    /// terminated when palantir is dropped. A stdlib mutex is used,
    /// as we need to access it from within drop. This must not be held
    /// across await points.
    join_set: std::sync::Mutex<JoinSet<()>>,
    /// Array of callbacks for when new peers connnect to us.
    /// A tokio mutex is used, as they will need to be held past await points.
    new_peer_callbacks: Mutex<Vec<Box<dyn Fn(String) + Send + Sync + 'static>>>,
    _phantom: PhantomData<V>
}

impl<V> Palantir<V> {
    /// # [`Palantir::new`]
    /// Creates a new palantir instance that will listen on the given port when run.
    pub fn new(port: u16) -> Self {
        Self {
            listen_port: port,
            peers: Default::default(),
            join_set: Default::default(),
            new_peer_callbacks: Default::default(),
            _phantom: PhantomData
        }
    }

    /// # [`Palantir::on_new_connection`]
    /// Registers a new callback for a new connection from a peer.
    /// Provides the peer's id to the callback
    pub async fn on_new_connection<F: Fn(String) + Send + Sync + 'static>(&self, callback: F) {
        // Lock the callback mutex
        let mut callbacks = self.new_peer_callbacks.lock().await;

        // Add the callback
        callbacks.push(Box::new(callback));
    }

    
}

impl<V: Validator> Palantir<V> {
    /// # [`Palantir::add_peer`]
    /// Connects to a new peer, sending the given validation.
    /// If the peer successfully connects, adds it to the map of known peers,
    /// returning its name.
    pub async fn new_peer<I: TryInto<IpAddr>>(&self, address: I, validation: V::Validation) {
        todo!();
    }

    /// # [`Palantir::run`]
    /// Runs Palantir's main loop for listening for incoming connections
    /// and dispatching messages. Because this method needs to
    /// dispatch messages back to [`fluxion`], this method
    /// takes a [`Fluxion`] instance.
    pub async fn run(self: Arc<Self>, system: Fluxion<Arc<Self>>, validator: Arc<V>) -> Result<(), PalantirError> {

        // Create the server endpoint
        let endpoint = Endpoint::server(
            ServerConfig::builder()
                .with_bind_default(self.listen_port)
                .with_identity(Identity::self_signed(["localhost","127.0.0.1"]).expect("key generation shouldn't fail"))
                .build()
        ).map_err(|_| PalantirError::UnableToInitializeServer)?;



        // Accept connections in a loop
        loop {
            // Accept the next incoming connection attempt
            let incoming = endpoint.accept().await;

            // Validate it
            if !validator.validate_incoming_session(&incoming).await {
                incoming.refuse();
                continue;
            }

            // Accept the incoming session request if valid
            let Ok(request) = incoming.await else {
                // Do not propagate io errors, as they can be synthesized
                // by a malicious peer.
                continue;
            };

            // Validate the request
            if !validator.validate_session_request(&request).await {
                request.forbidden().await;
                continue;
            }

            // Accept the connection if valid.
            let Ok(connection) = request.accept().await else {
                // Again, do not propagate io errors.
                continue;
            };

            // Spawn a task to handle the connection
            self.join_set.lock().expect("join set lock shouldn't be poisoned")
                .spawn(self.clone().handle_connection(connection, system.clone(), validator.clone()));

        }
    }


    /// # [`Palantir::handle_connection`]
    /// Handles a new connection from a peer, specifically accepting new channels.
    async fn handle_connection(self: Arc<Self>, connection: Connection, system: Fluxion<Arc<Self>>, validator: Arc<V>) {
        todo!()
    }
}



impl<V: Send + Sync + 'static> Delegate for Palantir<V> {
    async fn get_actor<A: fluxion::Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>> 
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {
        todo!()
    }
}