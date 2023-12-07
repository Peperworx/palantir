//! Palantir needs both x509 Certificates and matching ecdsa keypairs.
//! Additionally, palantir requires that NistP384 is used.
//! This module contains a struct that wraps a x509 certificate,
//! and extracts its NistP384 key for use.

use ecdsa::{SigningKey, VerifyingKey};
use elliptic_curve::{SecretKey, PublicKey};
use p384::NistP384;

use rustls_pki_types::CertificateDer;
use thiserror::Error;



/// # [`Certificate`]
/// This struct contains both a NistP384 keypair
/// and an x509 certificate.
/// This struct acts as an intermediate step between rcgen, rustls, and ecdsa crates.
#[derive(Clone)]
pub struct Certificate {
    /// The keypair
    keypair: (SecretKey<NistP384>, PublicKey<NistP384>),
    /// The x509 certificate, encoded as DER bytes, which contains the same
    /// secret key as the above keypair
    certificate: Vec<u8>,
}

impl Certificate {
    /// Gets the signing key
    pub fn signing_key(&self) -> SigningKey<NistP384> {
        self.keypair.0.into()
    }

    /// Gets the verifying key
    pub fn verifying_key(&self) -> VerifyingKey<NistP384> {
        self.keypair.1.into()
    }

    /// Gets the rustls certificate
    pub fn certificate(&self) -> rustls_pki_types::CertificateDer {
        rustls_pki_types::CertificateDer::try_from(self.certificate).unwrap()
    }
}


#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("needs a p384 certificate")]
    NeedsP384,
    #[error("Invalid certificate")]
    InvalidCertificate,
}


impl TryFrom<Vec<u8>> for Certificate {
    type Error = ConversionError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {

        // Create a rcgen Certificate
        let cert = CertificateDer::try_from(value.as_ref())
            .map_err(|_| ConversionError::InvalidCertificate)?;
        

        // Get the secret key
        let sk = SecretKey::<NistP384>::from_sec1_der(
            &cert
        ).map_err(|_| ConversionError::NeedsP384)?;

        // Derive the public key
        let pk = sk.public_key();

        Ok(Self {
            keypair: (sk, pk),
            certificate: value,
        })
    }
}