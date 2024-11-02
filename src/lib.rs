//! # Palantir

#[warn(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]

pub mod error;
pub mod validation;
mod frame;
mod message;

use std::{collections::HashMap, marker::PhantomData, net::IpAddr, sync::Arc};

use error::{PalantirError, TransmissionError};
use fluxion::{Delegate, Fluxion, Identifier, IndeterminateMessage, MessageSender};
use frame::Framed;
use message::PalantirMessage;
use tokio::{sync::Mutex, task::JoinSet};
use validation::Validator;
use wtransport::{endpoint::ConnectOptions, tls::client::NoServerVerification, ClientConfig, Connection, Endpoint, Identity, ServerConfig};


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
    peers: Mutex<HashMap<String, Connection>>,
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
    pub async fn new_peer<S: ToString>(&self, address: S, validation: V::Validation, headers: &HashMap<String, String>) -> Result<(), PalantirError> {
        
        // Create the client endpoint
        let endpoint = Endpoint::client(
            ClientConfig::builder()
                .with_bind_default()
                .with_custom_tls(
                    wtransport::tls::rustls::ClientConfig::builder()
                        .dangerous()
                        .with_custom_certificate_verifier(Arc::new(NoServerVerification::new()))
                        .with_no_client_auth()
                )
                .build()
        ).map_err(|e| PalantirError::ClientConnectingError(e.into()))?;

        // Build connection options with headers
        let mut opts = ConnectOptions::builder(address)
            .add_header("name",&self.name);

        for (key, value) in headers.iter() {
            opts = opts.add_header(key, value)
        }        

        // Connect to the server
        let conn = endpoint.connect(
            opts
        ).await.map_err(
            |e| PalantirError::ClientConnectingError(e.into())
        )?;

        // Now we need to run the handshake.


        // First, open a bidirectional channel just for the handshake
        let (send, recv) = conn.open_bi().await.map_err(
            |e| PalantirError::ConnectionError(e.into())
        )?.await.map_err(|e| PalantirError::TransmissionError(e.into()))?;

        // Wrape it in packet framing.
        let mut framed = Framed::<PalantirMessage<V::Validation>>::new(send, recv);
        
        // Construct and send the client handshake request
        framed.send(&PalantirMessage::ClientHandshake {
            magic: ['P', 'A', 'L', 'A', 'N', 'T', 'I', 'R'],
            name: self.name.clone(),
            validation
        }).await?;


        // Wait for the server's response.
        let PalantirMessage::ServerResponse { magic, name } = framed.recv().await? else {
            // Send the peer a message indicating it failed and return an error
            framed.send(&PalantirMessage::UnexpectedPacket).await?;
            return Err(PalantirError::PeerIncorrectlyResponded);
        };

        // If the magic is wrong, the data is malformed, most likely because the peer
        // is not a palantir server.
        if magic != ['P', 'A', 'L', 'A', 'N', 'T', 'I', 'R'] {
            framed.send(&PalantirMessage::MalformedData).await?;
            return Err(PalantirError::PeerIncorrectlyResponded);
        }

        // Now we have the peer's name.
        // Check if it exists
        if let Some(peer) = self.peers.lock().await.get(&name) {
            // If it does, check the addresses.
            // If they match, noop
            if peer.remote_address() == conn.remote_address() {
                return Ok(());
            }

            // Otherwise, error
            return Err(PalantirError::DuplicateName);
        }

        // If not, lets add it
        self.peers.lock().await.insert(name, conn);

        // The peer has been added.
        Ok(())
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
        )?;



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

            // The name header must be set. This is the name that the other peer is requesting.
            let Some(name) = request.headers().get("name") else {
                request.forbidden().await;
                continue;
            };

            // If the name already exists, deny the peer
            if self.peers.lock().await.contains_key(name) {
                request.forbidden().await;
                continue;
            }

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
                .spawn(self.clone().handle_connection(connection, system.clone(), validator.clone(), name));

        }
    }


    /// # [`Palantir::handle_connection`]
    /// Handles a new connection from a peer, specifically accepting new channels.
    async fn handle_connection(self: Arc<Self>, connection: Connection, system: Fluxion<Arc<Self>>, validator: Arc<V>, name: String) {
        
        


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

        



        
    }
}



impl<V: Send + Sync + 'static> Delegate for Palantir<V> {
    async fn get_actor<A: fluxion::Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>> 
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {
        todo!()
    }
}