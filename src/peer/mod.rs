//! # Peer
//! This module provides [`Peer`] and [`Channel`], which provide basic peer-to-peer and request/response semantics
//! on top of WebTransport.

use std::{collections::HashMap, sync::Arc};

use channel::Channel;
use error::{AddPeerError, OpenChannelError};
use message::{ActorID, PeerMessage};
use tokio::{sync::mpsc, task::JoinSet};
use wtransport::{proto::frame::FrameKind, Connection};

use crate::frame::Framed;

pub mod error;
pub mod channel;
mod message;



/// # [`RequestHandler`]
/// A request handler is a type that is attached to a peer to handle incoming requests from other peers.
pub trait RequestHandler {
    /// # [`RequestHandler::handle_request`]
    /// Called with the request body data and the name of the sender
    /// whenever a new request is received.
    /// 
    /// # Errors
    /// Returns a user-defined error code whenever an error occurs.
    async fn handle_request(&self, data: Vec<u8>, peer: &str) -> Result<Vec<u8>, u32>;
}


/// # [`Peer`]
/// A peer serves primarily to manage connections with other peers and provide communication channels to them upon request.
pub struct Peer<V, H> {
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
    /// The request handler is stored similarly to the validator
    request_handler: H,
}

impl<V, H> Peer<V, H> {

    /// # [`Peer::new`]
    /// Creates a new peer that is configured to listen on the given port
    pub fn new(listen_port: u16, name: String, validator: V, request_handler: H) -> Self {
        Self {
            listen_port,
            name,
            join_set: Default::default(),
            peers: Default::default(),
            validator,
            request_handler
        }
    }

    /// # [`Peer:add`]
    /// Connects to and adds the peer with the given address.
    /// Returns the peer's name.
    pub async fn add<S: ToString>(&self, address: S) -> Result<String, AddPeerError> {
        todo!()
    }

    /// # [`Peer::open_channel`]
    /// Opens a channel to the given peer with the given ID, requesting the given actor ID
    /// as the channel's target.
    /// 
    /// # Errors
    /// Returns an error if the peer doesn't exist, or if there was a problem establishing the channel.
    /// 
    /// # Panics
    /// This function may panic if the peers mutex is poisoned, which should almost never happen.
    /// If it does, it means that the only current option is to crash.
    pub async fn open_channel(&self, peer: &str, id: ActorID) -> Result<Channel, OpenChannelError> {

        // Try to retrieve the connection
        let conn = self.peers.read().expect("peer lock shouldn't be poisoned")
            .get(peer).ok_or(OpenChannelError::PeerDoesntExist)?.clone();

        // Open a new bidi channel
        let (send, recv) = conn.open_bi().await
            .map_err(|e| OpenChannelError::ConnectionError(e.into()))?.await
            .map_err(|e| OpenChannelError::TransmissionError(e.into()))?;

        // Wrap in a framed packet sender
        let mut framed = Framed::<PeerMessage>::new(send, recv);

        // Send our intent to use this as a request/response channel
        framed.send(&PeerMessage::Initialize(message::ChannelPurpose::RequestResponse(id))).await?;


        // Wrap in a channel
        let channel = Channel::new(framed.0);

        // Create the future for the channel's main loop
        let channel_future = channel.create_run_future(framed.1);

        // Spawn the channel's main loop on the join set
        self.join_set.lock().expect("join set lock shouldn't be poisoned")
            .spawn(channel_future);

        // Return the channel
        Ok(channel)
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