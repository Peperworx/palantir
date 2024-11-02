//! # Palantir

#[warn(clippy::pedantic)]
#[allow(clippy::module_name_repetitions)]

pub mod error;
pub mod validation;
mod frame;
mod message;

use std::{collections::HashMap, future::Future, marker::PhantomData, net::IpAddr, pin::Pin, sync::Arc};

use error::{FramedError, HandshakeError, PalantirError, TransmissionError};
use fluxion::{Delegate, Fluxion, Identifier, IndeterminateMessage, MessageSender};
use frame::Framed;
use message::{handshake, PalantirMessage, Side};
use tokio::{sync::Mutex, task::JoinSet};
use validation::Validator;
use wtransport::{endpoint::ConnectOptions, tls::client::NoServerVerification, ClientConfig, Connection, Endpoint, Identity, ServerConfig};

/// # [`CallbackFuture`]
/// Pinned future that is returned by callbacks.
type CallbackFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

/// # [`AsyncCallback`]
/// Represents a boxed callback that returns a [`CallbackFuture`]
/// and accepts a reference to a generic argument.
type AsyncCallback<V> = Box<dyn Fn(&V) -> CallbackFuture<'_> + Send + Sync + 'static> ;

/// # [`Palantir`]
/// Palantir enables rudimentary networking for [`fluxion`] via [`wtransport`].
pub struct Palantir<V> {
    /// The port that Palantir will listen on when [`Palantir::run`]
    /// is called. This will be a UDP port, as Palantir uses WebTransport.
    listen_port: u16,
    /// This peer's name
    name: String,
    /// This palantir instance "owns" two types of tasks:
    /// 1. Those spawned by the main loop to handle incoming connections
    /// 2. Those spawned by the previous tasks that handle incoming channels.
    /// 
    /// We do *not* want these tasks to be orphaned when the Palantir instance
    /// is dropped, so a join set is used to ensure that all tasks are aborted
    /// when this instance is dropped. A standard library mutex is used,
    /// as the value will be write-heavy, and  needs to be mutably accessed
    /// from the [`Drop`] trait,which is not asynchronous.
    /// This means that a mutex guard for this join set
    /// *MUST NOT* be held across an await point.
    /// 
    /// # Notes on Error Handling
    /// The way errors are handled in palantir is a little complex,
    /// as it is possible for multiple errors to cause another.
    /// This is currently handled by tasks returning a vec of errors.
    /// In the future, this may be handled by using some more complex error types,
    /// but at the moment this solution works.
    join_set: std::sync::Mutex<JoinSet<Result<(), Vec<PalantirError>>>>,
    /// A mapping of peer IDs to [`Connection`] objects.
    /// 
    /// The [`wtransport::Connection`] objects stored by this 
    /// field do not require mutable access for operations.
    /// As such, they are stored in an [`Arc`] that can be cloned
    /// out to threads that require access. This can both greatly
    /// simplify code, and possibly lead to improved performance,
    /// as there is no need to acquire a lock every use after the initial retrieval.
    /// Because accesses will be read-heavy, a standard library read-write lock
    /// is used for synchronization. The guard *MUST NOT* be held
    /// across any await points. Asynchronous use of the contained [`Connection`]
    /// should first clone the [`Arc`], and then drop the guard.
    peers: std::sync::RwLock<HashMap<String, Arc<Connection>>>,
    /// Array of callbacks for when new peers connnect to us.
    /// 
    /// Almost all accesses to this method will be mutable, so
    /// a mutex is used. As this will need to be held across
    /// await points when the callbacks are called, a tokio
    /// mutex is used. See the [`AsyncCallback`] type for more details.
    new_peer_callbacks: Mutex<Vec<AsyncCallback<str>>>,
    /// The validator is stored as a part of the palantir instance,
    /// and is shared between every task. It is only provided
    /// immutable access to itself, and as such doesn't need any
    /// synchronization primitives.
    validator: V,
}

impl<V> Palantir<V> {
    /// # [`Palantir::new`]
    /// Creates a new palantir instance that will listen on the given port when run.
    pub fn new(port: u16, name: String, validator: V) -> Self {
        Self {
            listen_port: port,
            peers: Default::default(),
            join_set: Default::default(),
            new_peer_callbacks: Default::default(),
            name,
            validator,
        }
    }

    /// # [`Palantir::on_new_connection`]
    /// Registers a new callback for a new connection from a peer.
    /// Provides the peer's id to the callback
    pub async fn on_new_connection<F: Fn(&str) -> CallbackFuture + Send + Sync + 'static>(&self, callback: F) {
        // Lock the callback mutex
        let mut callbacks = self.new_peer_callbacks.lock().await;

        // Add the callback
        callbacks.push(Box::new(callback));
    }

    
}

impl<V: Validator> Palantir<V> {
    

    /// # [`Palantir::handle_connection`]
    /// This future is spawned as a new task whenever a new connection
    /// is created.
    pub async fn handle_connection(self: Arc<Self>, connection: Arc<Connection>, mut state: V::State, side: Side) -> Result<(), Vec<PalantirError>> {

        // Handle the handshake on this connection, returning the name of the given peer
        let name = handshake(
            &self,
            &connection,
            &mut state,
            side
        ).await.map_err(|e| e.into_iter().map(PalantirError::from).collect::<Vec<_>>())?;


        // Handle the callbacks for a new connection
        let callbacks = self.new_peer_callbacks.lock().await ;
        
        for cb in callbacks.iter() {
            cb(&name).await;
        }

        

       
        // In a loop, accept new channels
        loop {

            // Accept the next channel
            let (send, recv) = connection.accept_bi().await
                .map_err(|e| vec![PalantirError::ConnectionError(e.into())])?;

            // Wrap in packet framing
            let mut framed = Framed::<PalantirMessage<V>>::new(send, recv);

            

        };
    }
}



impl<V: Send + Sync + 'static> Delegate for Palantir<V> {
    async fn get_actor<A: fluxion::Handler<M>, M: IndeterminateMessage>(&self, id: Identifier<'_>) -> Option<Arc<dyn MessageSender<M>>> 
        where M::Result: serde::Serialize + for<'a> serde::Deserialize<'a> {
        todo!()
    }
}