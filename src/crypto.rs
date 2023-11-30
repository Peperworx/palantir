//! Compatibility layers for NistP384 elliptic curve keypairs.


use elliptic_curve::{PublicKey, SecretKey, pkcs8::EncodePrivateKey};
use p384::NistP384;
use rand_core::CryptoRngCore;
use rcgen::DistinguishedName;


/// Stores a public and private p384 key.
pub struct KeyPair(SecretKey<NistP384>, PublicKey<NistP384>);


impl KeyPair {
    // Generates a new, random keypair from a RNG
    pub fn random(rng: &mut impl CryptoRngCore) -> Self {

        // Generate the secret key
        SecretKey::random(rng).into()
    }

    /// Gets the ecdsa signing key
    pub fn signing_key(&self) -> ecdsa::SigningKey<NistP384> {
        self.0.clone().into()
    }

    /// Gets the ecdsa verifying key
    pub fn verifying_key(&self) -> ecdsa::VerifyingKey<NistP384> {
        self.1.clone().into()
    }

    /// Create an x509 CA from this certificate, using the given distinguished name
    /// and alt names. Requires the PKCS_ECDSA_P384_SHA384 algorithm
    pub fn to_x509_ca(&self, distinguished_name: DistinguishedName, subject_alt_names: impl Into<Vec<String>>) -> rcgen::Certificate {
        

        // Default params
        let mut cert_params = rcgen::CertificateParams::new(subject_alt_names);

        // PKCS_ECDSA_P384_SHA384
        cert_params.alg = &rcgen::PKCS_ECDSA_P384_SHA384;

        // Distinguished name
        cert_params.distinguished_name = distinguished_name;

        // Set it as a CA
        cert_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        // Set the keypair
        cert_params.key_pair = Some(self.try_into().unwrap());

        // Generate the certificate
        rcgen::Certificate::from_params(cert_params).unwrap()
    }

    /// Given an x509 CA certificate, a distinguished name, and some alt names, return a
    /// DER-serialized certificate for this keypair
    pub fn to_certificate(&self, ca: &rcgen::Certificate, distinguished_name: DistinguishedName, subject_alt_names: impl Into<Vec<String>>) -> Vec<u8> {


        // Default params
        let mut cert_params = rcgen::CertificateParams::new(subject_alt_names);

        // PKCS_ECDSA_P384_SHA384
        cert_params.alg = &rcgen::PKCS_ECDSA_P384_SHA384;

        // Distinguished name
        cert_params.distinguished_name = distinguished_name;

        // Set it as a CA
        cert_params.is_ca = rcgen::IsCa::ExplicitNoCa;

        // Set the keypair
        cert_params.key_pair = Some(self.try_into().unwrap());

        // Generate the certificate
        let cert = rcgen::Certificate::from_params(cert_params).unwrap();

        // Serialize and sign
        cert.serialize_der_with_signer(ca).unwrap()
    }
}


impl From<SecretKey<NistP384>> for KeyPair {
    fn from(secret_key: SecretKey<NistP384>) -> Self {
        
        // Get the public key
        let public_key = secret_key.public_key();

        Self(secret_key, public_key)
    }
}

#[derive(Debug)]
pub struct ConversionError;

impl TryFrom<KeyPair> for rcgen::KeyPair {
    type Error = ConversionError;

    fn try_from(value: KeyPair) -> Result<Self, Self::Error> {
        // First create a PCKS8 document
        let key_doc = value.signing_key().to_pkcs8_der()
            .map_err(|_| ConversionError)?;
        
        // serialize it to ANS.1 DER
        let key_der = key_doc.to_bytes();

        // Create a rcgen::KeyPair
        rcgen::KeyPair::from_der(&key_der)
            .map_err(|_| ConversionError)
    }
}


impl TryFrom<&KeyPair> for rcgen::KeyPair {
    type Error = ConversionError;

    fn try_from(value: &KeyPair) -> Result<Self, Self::Error> {
        // First create a PCKS8 document
        let key_doc = value.signing_key().to_pkcs8_der()
            .map_err(|_| ConversionError)?;
        
        // serialize it to ANS.1 DER
        let key_der = key_doc.to_bytes();

        // Create a rcgen::KeyPair
        rcgen::KeyPair::from_der(&key_der)
            .map_err(|_| ConversionError)
    }
}



