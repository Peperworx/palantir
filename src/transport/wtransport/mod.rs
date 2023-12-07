//! # Wtransport
//! Basic [`Transport`] that uses WebTransport as provided by the [`wtransport`] crate.
//! 
//! [`DirectPeer`] provides a [`Transport`] implementation that communicates between peers using direct connections.
//! It hosts a local WebTransport server, can initiate connections with other [`DirectPeer`]s given an IP address and port,
//! and can receive connections from other [`DirectPeer`]s.
//! 
//! A [`DirectPeer`] is identified and authenticated using ECDSA P-384.



mod session;
mod context;

use std::{net::{IpAddr, SocketAddr}, sync::Arc};

use futures::FutureExt;
use p384::pkcs8::EncodePrivateKey;
use crate::crypto::Certificate;

use serde::de;
use wtransport::{ServerConfig, endpoint::IncomingSession};

use crate::PeerId;

use super::TransportError;

/// # [`DirectPeer`]
/// Implements [`Transport`] for communicating between directly known peers over webtransport.
/// This struct is safe to clone around, and behaviour will be as expected. However, it should be
/// noted that [`DirectPeer::run`] should only ever be called once, as cloned [`DirectPeer`]s still
/// use the same channels.
#[derive(Clone)]
pub struct DirectPeer {
    /// The address and port to listen on
    addr: SocketAddr,
    
    /// This peer's ID, as calculated from its VerifyingKey
    id: PeerId,
    /// This peer's x509 certificate
    cert: Certificate,
    /// The TLS configuration
    tls_config: rustls::ServerConfig,
    /// The sender for the communication channel
    /// used to communicate with the task running the server.
    sender: kanal::AsyncSender<()>,
    /// The receiver for the communication channel.
    /// This should only be accessed by one task.
    receiver: kanal::AsyncReceiver<()>,
}


impl DirectPeer {

    /// Creates a new [`DirectPeer`], configuring it to listen on the given address
    /// using the given x509 certificate
    pub fn new(
            address: SocketAddr, 
            cert: Certificate) -> Result<Self, TransportError> {
        
        

        // Create an ID for this peer from the Verifying Key
        let id = PeerId::from(cert.verifying_key());
        
        // Create a channel for the transport to communicate with the running task
        let (sender, receiver) = kanal::unbounded_async();
        

        // Create the tls config
        let tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![cert.certificate()],
                rustls_pki_types::PrivateKeyDer::Pkcs8(
                    rustls_pki_types::PrivatePkcs8KeyDer::from(
                        cert.signing_key().to_pkcs8_der().unwrap().as_bytes()
                    )
                )
            ).unwrap();

        Ok(Self {
            addr: address,
            id,
            cert, sender, receiver,
            tls_config,
        })
    }

    /// Runs a webtransport server, used to find peers and receive connections from them.
    pub async fn run(self) {

        // Create a new context to share data between subtasks,
        let context = context::DirectPeerContext {
            direct_peer: Arc::new(self.clone()),
            sender: self.sender.clone(),
        };

        

        // Create the server config
        let config = wtransport::ServerConfig::builder()
            .with_bind_address(self.addr)
            .with_custom_tls(self.tls_config)
            .build();

        // Create the server
        let server = wtransport::Endpoint::server(config).unwrap();
        

        // Main server loop
        loop {
            
            
            futures::select! {
                session = server.accept().fuse() => {
                    // If a peer requests an incoming session, start a new task to handle it
                    
                    

                    // Create the session handler
                    let sh = session::SessionHandler {
                        context: context.clone(),
                        session
                    };

                    // Begin execution of the session handler
                    tokio::spawn(async move {
                        println!("new session");
                        sh.run().await
                    });
                    
                },
                control_message = self.receiver.recv().fuse() => {

                }
            }
        }
    }

    /// Handles a local session
    async fn session_handler(&self, sender: kanal::AsyncSender<()>, session: IncomingSession, killed: kanal::OneshotAsyncReceiver<()>) {


        
    }
}