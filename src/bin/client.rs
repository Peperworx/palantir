use std::{sync::Arc, net::SocketAddr};

use ecdsa::{SigningKey, VerifyingKey};
use p384::pkcs8::EncodePrivateKey;
use palantir::{transport::wtransport::DirectPeer, PeerId};
use rand_core::OsRng;
use rcgen::DistinguishedName;
use rustls::{Certificate, client::ServerCertVerifier, RootCertStore};
use wtransport::{ClientConfig, Endpoint};


struct NoServerVerification;

impl ServerCertVerifier for NoServerVerification {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        server_name: &rustls::ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

#[tokio::main]
async fn main() {
    let sk = SigningKey::random(&mut OsRng);
    let vk = VerifyingKey::from(&sk);

    let address: SocketAddr = "0.0.0.0:4432".parse().unwrap();

    // First create a PCKS8 document
    let key_doc = sk.to_pkcs8_der().unwrap();
        
    // serialize it to ANS.1 DER
    let key_der = key_doc.to_bytes();

    // Create a rcgen::KeyPair
    let rcgen_key = rcgen::KeyPair::from_der(&key_der).unwrap();

    // Create an ID for this peer from the Verifying Key
    let id = PeerId::from(vk);
    
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
    let cert = rcgen::Certificate::from_params(cparams).unwrap();
    let cert = Certificate(cert.serialize_der().unwrap());

    // Create the tls config
    let tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(NoServerVerification))
        //.with_root_certificates(rustls::RootCertStore::empty())
        .with_client_auth_cert(vec![cert], rustls::PrivateKey(key_der.to_vec()))
        .unwrap()
    ;

    
    
    let config = ClientConfig::builder()
        .with_bind_default()
        .with_custom_tls(tls_config)
        .build();

    let connection = Endpoint::client(config).unwrap()
        .connect("https://127.0.0.1:4433")
        .await.unwrap();
    
    let mut buf = [0u8; 100];
    let len = connection.open_bi().await.unwrap().await.unwrap().1.read(&mut buf).await.unwrap().unwrap();
    println!("{:?}", String::from_utf8(buf[..len].to_vec()))
}