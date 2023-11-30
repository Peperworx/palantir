//! # Session
//! Internals to handle new sessions from peers.



use p384::pkcs8::DecodePublicKey;
use wtransport::endpoint::IncomingSession;

use crate::{transport::TransportError, PeerId};

use super::context::DirectPeerContext;


pub struct SessionHandler {
    /// The context used by this session handler
    pub(super) context: DirectPeerContext,
    /// The incoming session we are handling
    pub(super) session: IncomingSession,
}

impl SessionHandler {
    pub async fn run(self) -> Result<(), TransportError> {
        println!("in runner");
        // Get the session request
        let session = self.session.await.unwrap();
        println!("request accepted, getting headers");
        println!("{:?}", session.headers());
        let session = session.accept().await.unwrap();
        let mut conn = session.accept_bi().await.unwrap();
        conn.0.write(b"hello, world").await.unwrap();
        // The client identifies itself by sending an Authorization header
        // with auth scheme "key", followed by the client's public key PEM encoded.
        //let auth = session.headers().get("Authorization").ok_or(TransportError::AuthenticationFailed)?;
        //let mut split = auth.split(' ');
        //let scheme = split.next().ok_or(TransportError::AuthenticationFailed)?;
        //if scheme != "key" {
        //    return Err(TransportError::AuthenticationFailed);
        //}
        //let encoded = split.next().ok_or(TransportError::AuthenticationFailed)?;

        // Base 64 decode the encoded key
        //let keypair = ecdsa::VerifyingKey::<p384::NistP384>::from_public_key_pem(encoded)
        //    .map_err(|_| TransportError::AuthenticationFailed)?;

        // Create the peer id
        //let id = PeerId::from(&keypair);

        //println!("new peer with id {id:?}");

        // Report to user for authorization
        
        Ok(())
    }
}