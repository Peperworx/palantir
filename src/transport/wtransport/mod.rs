//! # Wtransport
//! Basic [`Transport`] that uses WebTransport as provided by the [`wtransport`] crate.
//! 
//! [`DirectPeer`] provides a [`Transport`] implementation that communicates between peers using direct connections.
//! It hosts a local WebTransport server, can initiate connections with other [`DirectPeer`]s given an IP address and port,
//! and can receive connections from other [`DirectPeer`]s.
//! 
//! A [`DirectPeer`] is identified and authenticated using ECDSA P-384.


use std::{net::{IpAddr, SocketAddr}, sync::Arc};

use p384::pkcs8::EncodePrivateKey;
use rcgen::DistinguishedName;
use rustls::Certificate;
use serde::de;
use wtransport::ServerConfig;

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
    /// The sender for the communication channel
    /// used to communicate with the task running the server.
    sender: kanal::AsyncSender<()>,
    /// The receiver for the communication channel.
    /// This should only be accessed by one task.
    receiver: kanal::AsyncReceiver<()>,
}


impl DirectPeer {

    /// Creates a new [`DirectPeer`], configuring it to listen on the given address
    /// using the given ECDSA keypair
    pub fn new(address: SocketAddr, keypair: (ecdsa::SigningKey<p384::NistP384>, ecdsa::VerifyingKey<p384::NistP384>)) -> Result<Self, TransportError> {
        // Convert our keypair to a rcgen::KeyPair
        
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

        // The alt name will be the address we are hosting on
        let alt_names = vec![address.ip().to_string()];

        

        // Certificate params
        let mut cparams = rcgen::CertificateParams::new(alt_names);
        
        // We want to use ECDSA P384
        cparams.alg = &rcgen::PKCS_ECDSA_P384_SHA384;
        // And set a distinguished name
        cparams.distinguished_name = distname;
        // This certificate can not sign other certificates
        cparams.is_ca = rcgen::IsCa::ExplicitNoCa;
        // Explicitly set its keypair so a new one is not generated.
        cparams.key_pair = Some(rcgen_key);

        // Create the certificate
        let cert = rcgen::Certificate::from_params(cparams).or(Err(TransportError::InvalidKey))?;
        let cert = Certificate(cert.serialize_der().or(Err(TransportError::InvalidKey))?);

        // Create a channel for the transport to communicate with the running task
        let (sender, receiver) = kanal::unbounded_async();
        
        Ok(Self {
            addr: address,
            keypair, id,
            cert, sender, receiver,
        })
    }

    /// Runs a webtransport server, used to find peers and receive connections from them.
    pub async fn run(self) {

        // Create the tls config
        let tls_config = rustls::ServerConfig::builder()
            .with_cipher_suites(rustls::DEFAULT_CIPHER_SUITES)
            .with_kx_groups(&[&rustls::kx_group::SECP384R1])
            .with_safe_default_protocol_versions().unwrap() // If the above code is properly formed, there should be no errors at this point
            .with_client_cert_verifier(Arc::new(rustls::server::AllowAnyAuthenticatedClient::new(rustls::RootCertStore::empty())))
            .with_single_cert(vec![self.cert], rustls::PrivateKey(self.keypair.0.to_pkcs8_der().unwrap().to_bytes().to_vec()))
            .unwrap();

        // Create the server config
        let config = wtransport::ServerConfig::builder()
            .with_bind_address(self.addr)
            .with_custom_tls(tls_config)
            .build();

        // Create the server
        let server = wtransport::Endpoint::server(config).unwrap();

        // In a loop wait for sessions
        loop {
            let session = server.accept().await;
        }
    }
}