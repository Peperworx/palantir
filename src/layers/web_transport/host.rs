//! # Host Layer
//! 
//! Contains a webtransport host that implements layer.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use crate::identification::{generate_random_id, HostedPeerID};

use super::peer::WTPeer;

use wtransport::endpoint::endpoint_side::Server;
use wtransport::endpoint::IncomingSession;
use wtransport::{Endpoint, ServerConfig};

use super::WTLayerError;


pub struct WTHost<P> {
    /// Set of connected clients
    pub clients: Arc<RwLock<HashMap<[u128; 2], Arc<WTPeer<P>>>>>,
}

impl<P: Send + Sync + 'static> WTHost<P> {
    /// Initialize the server and start a task listening for peers.
    pub async fn start(config: ServerConfig) -> Result<Self, WTLayerError> {

        // Create the server endpoint
        let server = Endpoint::server(config)?;

        // Create the map of clients
        let clients = Arc::new(RwLock::new(HashMap::new()));

        // Clone it for the task
        let clients_ext = clients.clone();

        // Spawn a task accepting clients and assigning them as peers
        tokio::spawn(Self::accept(server, clients_ext));

        Ok(Self {
            clients
        })
    }

    /// Accepts clients in a loop
    async fn accept(server: Endpoint<Server>, clients: Arc<RwLock<HashMap<[u128; 2], Arc<WTPeer<P>>>>>) {
        loop {
            // Accept the incoming session
            let incoming = server.accept().await;

            println!("incoming session");

            // Spawn a task handling the incoming session.
            Self::handle_incoming(incoming, clients.clone()).await;
        }
    }

    /// Handles an incoming session
    async fn handle_incoming(incoming: IncomingSession, clients: Arc<RwLock<HashMap<[u128; 2], Arc<WTPeer<P>>>>>) {
        
        // Begin accepting the session
        let Ok(session) = incoming.await else {
            println!("connecting error");
            return;
        };


        println!("incoming session: {:?}", session.origin());

        // Accept the connection
        let Ok(conn) = session.accept().await else {
            println!("connection error");
            return;
        };

        // Wrap it in a peer and assign it an ID
        let id = generate_random_id(uuid::Uuid::nil().as_u128());
        let peer = Arc::new(WTPeer::<P>::new(conn, HostedPeerID::Client(id)));

        // Add it to the hashmap
        let mut cli = clients.write().await;
        cli.insert(id, peer);

        println!("new peer: {}/{}", uuid::Uuid::from_u128(id[0]), uuid::Uuid::from_u128(id[1]))
        
    }
}