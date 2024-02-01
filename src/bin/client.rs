use std::sync::Arc;

use palantir::{identification::HostedPeerID, layers::{web_transport::{client::WTClient, WTNamespaceID}, Layer, Namespace, Peer}};
use wtransport::{endpoint::ConnectOptions, ClientConfig};

use rustls::client::danger::ServerCertVerifier;
use rustls::client::danger::{ServerCertVerified, HandshakeSignatureValid};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};


#[derive(Debug)]
struct NoServerVerification;

impl ServerCertVerifier for NoServerVerification {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _signature: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _signature: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![]
    }
}


#[tokio::main]
pub async fn main() {

    let mut client_conf = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoServerVerification))
        .with_no_client_auth();

    

    // Create a new WT client connecting to the local host
    let client = WTClient::<String>::connect(
        ConnectOptions::builder("https://[::1]:4433")
        .build(),
        ClientConfig::builder()
        .with_bind_default()
        .with_custom_tls(
            client_conf
                
        )
        .build()
    ).await.unwrap();

    // Get the host peer
    let host = client.get_peer(HostedPeerID::Host).unwrap();

    // Open a test namespace
    let mut ns = host.open_namespace(WTNamespaceID::Named("test".to_string())).await.unwrap();

    // Send some data over the namespace
    ns.send(&"Hello, World!".to_string()).await.unwrap();


}