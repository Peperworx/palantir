//! # Palantir

#[warn(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]

pub mod error;
pub mod validation;
mod frame;
mod message;

use std::{collections::HashMap, marker::PhantomData, net::IpAddr, sync::Arc};

use error::PalantirError;
use fluxion::{Delegate, Fluxion, Identifier, IndeterminateMessage, MessageSender};
use frame::Framed;
use message::PalantirMessage;
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
    /// This peer's name
    name: String,
    _phantom: PhantomData<V>
}

impl<V> Palantir<V> {
    /// # [`Palantir::new`]
    /// Creates a new palantir instance that will listen on the given port when run.
    pub fn new(port: u16, name: String) -> Self {
        Self {
            listen_port: port,
            peers: Default::default(),
            join_set: Default::default(),
            new_peer_callbacks: Default::default(),
            name,
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
        
        // Accept bidirectional channels in a loop
        loop {

            // Accept the next bidirectional channel.
            let Ok((send, recv)) = connection.accept_bi().await else {
                // Drop the client altogether if there is a connection error
                connection.close(0u32.into(), b"connection error");
                return;
            };

            // Wrap the bidirectional channel in packet framing
            let framed = Framed::<PalantirMessage<V::Validation>>::new(send, recv);

            // Spawn a new task to handle the channel.
            self.join_set.lock().expect("join set lock shouldn't be poisoned")
                .spawn(self.clone().handle_channel(framed, system.clone(), validator.clone()));

        }
    }

    /// # [`Palantir::handle_channel`]
    /// Handles a new channel from a peer, including the handshake
    async fn handle_channel(self: Arc<Self>, mut framed: Framed<PalantirMessage<V::Validation>>, system: Fluxion<Arc<Self>>, validator: Arc<V>) {

        // Receive the next packet from the client
        let Ok(next) = framed.recv().await else {
            // Send the client a message saying there was an invalid packet,
            // ignoring the returned result just in case it was a connection error.
            let _ = framed.send(&PalantirMessage::InvalidPacket).await;
            return;
        };

        // Unpack the client's handshake, or inform of an invalid packet.
        let PalantirMessage::ClientHandshake { magic, name, validation } = next else {
            // Ignore the result, as we are exiting anyways.
            let _ = framed.send(&PalantirMessage::InvalidPacket).await;
            return;
        };

        // If the magic value is wrong, return
        if magic != ['P', 'A', 'L', 'A', 'N', 'T', 'I', 'R'] {
            // Ignore the result, as we are exiting anyways.
            let _ = framed.send(&PalantirMessage::InvalidPacket).await;
            return;
        }

        // Run the validation
        if !validator.validate_validation(&validation, &name).await {
            // Ignore the result, as we are exiting anyways.
            let _ = framed.send(&PalantirMessage::ValidationFailed).await;
            return;
        }

        // Send the server's response
        framed.send(&PalantirMessage::ServerResponse {
            magic: ['P', 'A', 'L', 'A', 'N', 'T', 'I', 'R'],
            name: self.name.clone(),
        });

        // Wait for the client's response
        let Ok(next) = framed.recv().await else {
            let _ = framed.send(&PalantirMessage::InvalidPacket).await;
            return;
        };

        // If it's not a client response, then return and do nothing
        let PalantirMessage::ClientResponse = next else {
            return;
        };

        // Now the client is validated

        // Split off the sender and receiver.
        let (send, recv) = (framed.0, framed.1);

        // Wrap the sender in a mutex, as it will solely
        // be used to send responses, which whill be sent from other threads.
        // It is likely more efficient to just use an arc and mutex than
        // to use a channel and select here.
        let send = Arc::new(Mutex::new(send));

        loop {
            
            // Receive the next packet
            let Ok(next) = recv.recv().await else {

            };

            todo!()
        }
    }
}



impl<V: Send + Sync + 'static> Delegate for Palantir<V> {
    async fn get_actor<A: fluxion::Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>> 
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {
        todo!()
    }
}