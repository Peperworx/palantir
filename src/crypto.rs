//! Compatibility layers for NistP384 elliptic curve keypairs.

use elliptic_curve::{SecretKey, PublicKey};
use p384::NistP384;

use thiserror::Error;



/// # [`Certificate`]
/// This struct contains both a NistP384 keypair
/// and an x509 certificate.
/// This struct acts as an intermediate step between rcgen, rustls, and ecdsa crates.
pub struct Certificate {
    /// The keypair
    keypair: (SecretKey<NistP384>, PublicKey<NistP384>),
    /// The x509 certificate, encoded as DER bytes, which contains the same
    /// secret key as the above keypair
    certificate: Vec<u8>,
}

impl Certificate {

}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("needs a p384 certificate")]
    NeedsP384,
    #[error("Invalid certificate")]
    InvalidCertificate,
}

impl<'a> TryFrom<Certificate> for rustls_pki_types::CertificateDer<'a> {
    type Error = ConversionError;

    fn try_from(value: Certificate) -> Result<rustls_pki_types::CertificateDer<'a>, ConversionError> {
        Ok(rustls_pki_types::CertificateDer::from(
            value.certificate
        ))
    }   
}

impl TryFrom<rcgen::Certificate> for Certificate {
    type Error = ConversionError;

    fn try_from(value: rcgen::Certificate) -> Result<Self, Self::Error> {

        // Get the secret key
        let sk = SecretKey::<NistP384>::from_sec1_der(
            value.get_key_pair().serialized_der()
        ).map_err(|_| ConversionError::NeedsP384)?;

        // Derive the public key
        let pk = sk.public_key();

        Ok(Self {
            keypair: (sk, pk),
            certificate: value.serialize_der(),
            signer: None,
        })
    }
}