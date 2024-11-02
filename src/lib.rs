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
    
}



impl<V: Send + Sync + 'static> Delegate for Palantir<V> {
    async fn get_actor<A: fluxion::Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>> 
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {
        todo!()
    }
}