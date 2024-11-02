//! # Peer
//! This module provides [`Peer`] and [`Channel`], which provide basic peer-to-peer and request/response semantics
//! on top of WebTransport.

use channel::Channel;
use error::{AddPeerError, OpenChannelError};
use tokio::sync::mpsc;

pub mod error;
pub mod channel;


pub struct Peer {

}

impl Peer {

    /// # [`Peer::new`]
    /// Creates a new peer that is configured to listen on the given port
    pub fn new(port: u16) -> Self {
        todo!()
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