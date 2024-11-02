//! # Peer
//! This module provides [`Peer`] and [`Channel`], which provide basic peer-to-peer and request/response semantics
//! on top of WebTransport.

use std::{collections::HashMap, sync::Arc};

use channel::Channel;
use error::{AddPeerError, OpenChannelError};
use tokio::{sync::mpsc, task::JoinSet};
use wtransport::Connection;

pub mod error;
pub mod channel;
mod message;


pub struct Peer<V> {
    /// This is the UDP port this peer will listen on.
    listen_port: u16,
    /// This is the name of this peer
    name: String,
    /// This peer instance "owns" several types of tasks.
    ///
    /// We do *not* want these tasks to be orphaned when the Palantir instance
    /// is dropped, so a join set is used to ensure that all tasks are aborted
    /// when this instance is dropped. A standard library mutex is used,
    /// as the value will be write-heavy, and  needs to be mutably accessed
    /// from the [`Drop`] trait,which is not asynchronous.
    /// This means that a mutex guard for this join set
    /// *MUST NOT* be held across an await point.
    join_set: std::sync::Mutex<JoinSet<()>>,
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
    /// The validator is stored as a part of the peer instance,
    /// and is shared between every task. It is only provided
    /// immutable access to itself, and as such doesn't need any
    /// synchronization primitives.
    validator: V,
}

impl<V> Peer<V> {

    /// # [`Peer::new`]
    /// Creates a new peer that is configured to listen on the given port
    pub fn new(listen_port: u16, name: String, validator: V) -> Self {
        Self {
            listen_port,
            name,
            join_set: Default::default(),
            peers: Default::default(),
            validator
        }
    }

    /// # [`Peer:add`]
    /// Connects to and adds the peer with the given address.
    /// Returns the peer's name.
    pub async fn add<S: ToString>(&self, address: S) -> Result<String, AddPeerError> {
        todo!()
    }

    /// # [`Peer::open_channel`]
    /// Opens a channel to the given peer with the given ID.
    pub async fn open_channel(&self, peer: &str) -> Result<Channel, OpenChannelError> {
        todo!()
    }

    /// # [`Peer:run`]
    /// Runs the peer's main loop in a separate task, returning the channel over which new channels are sent.
    pub async fn run(&self) -> mpsc::Receiver<Channel> {
        todo!()
    }

    /// # [`Peer:join`]
    /// Joins on all tasks owned by the peer.
    pub async fn join(&self) {
        todo!()
    }
}