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
use rcgen::DistinguishedName;
use rustls::Certificate;
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
    /// The ECDSA Signing and Verification keys used by this peer
    keypair: (ecdsa::SigningKey<p384::NistP384>, ecdsa::VerifyingKey<p384::NistP384>),
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
    /// using the given certificate
    pub fn new(
            address: SocketAddr, 
            cert: Vec<u8>) -> Result<Self, TransportError> {
        

        // First create a PCKS8 document
        let key_doc = keypair.0.to_pkcs8_der().or(Err(TransportError::InvalidKey))?;
        
        // serialize it to ANS.1 DER
        let key_der = key_doc.to_bytes();

        // Create a rcgen::KeyPair
        let rcgen_key = rcgen::KeyPair::from_der(&key_der).or(Err(TransportError::InvalidKey))?;

        // Create an ID for this peer from the Verifying Key
        let id = PeerId::from(keypair.1);
        
        // Build a x509 certificate

        // Create the distinguished name for this peer.
        // This will just contain the keypair as hex bytes
        let mut distname = DistinguishedName::new();
        distname.push(rcgen::DnType::CommonName, id.to_string());


        

        // Certificate params
        let mut cparams = rcgen::CertificateParams::new(vec![id.to_string()]);
        
        // We want to use ECDSA P384
        cparams.alg = &rcgen::PKCS_ECDSA_P384_SHA384;
        // And set a distinguished name
        cparams.distinguished_name = distname;
        // This certificate can not sign other certificates
        cparams.is_ca = rcgen::IsCa::NoCa;
        // Explicitly set its keypair so a new one is not generated.
        cparams.key_pair = Some(rcgen_key);

        // Create the certificate
        let cert = rcgen::Certificate::from_params(cparams).or(Err(TransportError::InvalidKey))?;
        let cert = Certificate(cert.serialize_der().or(Err(TransportError::InvalidKey))?);

        // Create a channel for the transport to communicate with the running task
        let (sender, receiver) = kanal::unbounded_async();
        
        // Create the tls config
        let tls_config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            
            .with_no_client_auth()
            .with_single_cert(vec![cert.clone()], rustls::PrivateKey(key_der.to_vec()))
            .unwrap();

        Ok(Self {
            addr: address,
            keypair, id,
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