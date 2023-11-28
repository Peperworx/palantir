//! # Wtransport
//! Basic [`Transport`] that uses WebTransport as provided by the [`wtransport`] crate.
//! 
//! [`DirectPeer`] provides a [`Transport`] implementation that communicates between peers using direct connections.
//! It hosts a local WebTransport server, can initiate connections with other [`DirectPeer`]s given an IP address and port,
//! and can receive connections from other [`DirectPeer`]s.
//! 
//! A [`DirectPeer`] is identified and authenticated using ECDSA P-384.


use std::net::IpAddr;

use rcgen::DistinguishedName;
use serde::de;

use crate::PeerId;


pub struct DirectPeer {}


impl DirectPeer {

    /// Creates a new [`DirectPeer`], configuring it to listen on the given address
    /// using the given ECDSA keypair
    pub fn new(address: IpAddr, keypair: (ecdsa::SigningKey<p384::NistP384>, ecdsa::VerifyingKey<p384::NistP384>)) -> Self {
        // Create an ID for this peer from the VerifyingKey
        let id = PeerId::from(keypair.1);
        
        // Build a x509 certificate

        // Create the distinguished name for this peer.
        // This will just contain the keypair as hex bytes
        let distname = DistinguishedName::new();
        distname.push(rcgen::DnType::CommonName, id.to_string());
        
        // Certificate params
        let cparams = rcgen::CertificateParams {
            /// ECDSA P384
            alg: &rcgen::PKCS_ECDSA_P384_SHA384,
            /// We don't care about expiry
            not_before: date_time_ymd(1975, 01, 01),
            not_after: date_time_ymd(4096, 01, 01),
            // No serial number
            serial_number: None,
            subject_alt_names: Vec::new(),
            distinguished_name: distname,
            is_ca: rcgen::IsCa::ExplicitNoCa,
            ..Default::default()
        };
        todo!()
    }
}