//! # Message
//! The data packet type that palantir sends over the wire, serialized using [`postcard`].

use serde::{Deserialize, Serialize};
use wtransport::Connection;

use crate::{
    error::{FramedError, HandshakeError, PalantirError, TransmissionError},
    frame::Framed,
    validation::Validator,
    Palantir,
};

/// # [`handshake`]
/// Handles the handshake over a given connection,
/// returning the name of the peer.
pub(crate) async fn handshake<V: Validator>(
    instance: &Palantir<V>,
    connection: &Connection,
    state: &mut V::State,
    side: Side,
) -> Result<String, Vec<HandshakeError>> {
    // Handle the connection differently for client vs server
    match side {
        Side::Initiator => client_handshake(instance, connection, state).await,
        Side::Acceptor => server_handshake(instance, connection, state).await,
    }
}

/// # [`client_handshake`]
/// Handles the handshake from the client's side.
pub(crate) async fn client_handshake<V: Validator>(
    instance: &Palantir<V>,
    connection: &Connection,
    state: &mut V::State,
) -> Result<String, Vec<HandshakeError>> {
    // The client always initializes the handshake.
    // So lets open a channel just for the handshale
    let (send, recv) = connection
        .open_bi()
        .await
        .map_err(|v| vec![HandshakeError::ConnectionError(v.into())])?
        .await
        .map_err(|v| vec![HandshakeError::TransmissionError(v.into())])?;

    // And wrap the channel in packet framing
    let mut framed = Framed::new(send, recv);

    // Now for the handshake

    // # Step 1
    // Client sends initiation to server

    // Construct the initiation
    let initiation = PalantirMessage::<V>::ClientInitiation {
        magic: "PALANTIR".to_string(),
        name: instance.name.clone(), // This instance's name
    };

    // Send the initiation
    framed.send(&initiation).await
        .map_err(|e| vec![e.into()])?;



    // # Step 2
    // Server sends response to client

    // This is where error handling might get a bit trickier.

    // Receive the server's response
    let server_response = framed.recv().await;

    let server_response = match server_response {
        Err(e @ FramedError::TransmissionError(_)) => {
            // Because this is the only error in the chain, we can just return a new vec.
            return Err(vec![e.into()]);
        }
        Err(e @ FramedError::InvalidEncoding { packet: _ }) => {
            // Create a vec with the error
            let mut errs: Vec<HandshakeError> = vec![e.into()];

            // Tell the peer it sent an invalid packet
            let res = framed.send(&PalantirMessage::MalformedData).await;

            // If there was another error, log it
            if let Err(e) = res {
                errs.push(e.into());
            }

            return Err(errs);
        }
        Err(_) => unreachable!("received packets can't exceed our default send size limit"),
        Ok(v) => v,
    };

    // Destructure the server's response
    let PalantirMessage::ServerResponse { magic, name } = server_response else {
        // Create the error containing the unexpected packet errror
        let mut errs = vec![HandshakeError::UnexpectedPacket];

        // Tell the peer it sent an unexpected packet
        let res = framed.send(&PalantirMessage::UnexpectedPacket).await;

        // If there was another error, log it
        if let Err(e) = res {
            errs.push(e.into());
        }

        return Err(errs);
    };

    // Validate the magic
    if magic != "PALANTIR" {
        let mut errs = vec![HandshakeError::InvalidMagic];

        // Tell the peer it sent an invalid magic value
        let res = framed.send(&PalantirMessage::InvalidMagic).await;

        // If there was another error, log it
        if let Err(e) = res {
            errs.push(e.into());
        }

        return Err(errs);
    }


    // Check if the name exists. If it does, error.
    if instance.peers.read().expect("peers map not poisoned").contains_key(&name) {
        let mut errs = vec![HandshakeError::NameTaken];

        // Tell the peer its name is taken
        let res = framed.send(&PalantirMessage::NameTaken).await;

        // If there was another error, log it
        if let Err(e) = res {
            errs.push(e.into());
        }

        return Err(errs);
    }



    // # Step 3
    // Run validation
    instance.validator.handshake(&mut framed, state, &name).await?;

    // # Step 4
    // Send confirmation of finished handshake
    framed.send(&PalantirMessage::HandshakeCompleted).await
        .map_err(|e| vec![e.into()])?;

    // # Step 5
    // Close the channel.
    framed.0.close().await;

    Ok(name)
}



/// # [`server_handshake`]
/// Handles the handshake from the server's side.
pub(crate) async fn server_handshake<V: Validator>(
    instance: &Palantir<V>,
    connection: &Connection,
    state: &mut V::State,
) -> Result<String, Vec<HandshakeError>> {
    // The client always initializes the handshake.
    // So lets open a channel just for the handshale
    let (send, recv) = connection
        .open_bi()
        .await
        .map_err(|v| vec![HandshakeError::ConnectionError(v.into())])?
        .await
        .map_err(|v| vec![HandshakeError::TransmissionError(v.into())])?;

    // And wrap the channel in packet framing
    let mut framed = Framed::new(send, recv);

    // Now for the handshake

    // # Step 1
    // Client sends initiation to server

    // Recieve the initiation
    let initiation = framed.recv().await;

    let initiation = match initiation {
        Err(e @ FramedError::TransmissionError(_)) => {
            // Because this is the only error in the chain, we can just return a new vec.
            return Err(vec![e.into()]);
        }
        Err(e @ FramedError::InvalidEncoding { packet: _ }) => {
            // Create a vec with the error
            let mut errs: Vec<HandshakeError> = vec![e.into()];

            // Tell the peer it sent an invalid packet
            let res = framed.send(&PalantirMessage::MalformedData).await;

            // If there was another error, log it
            if let Err(e) = res {
                errs.push(e.into());
            }

            return Err(errs);
        }
        Err(_) => unreachable!("received packets can't exceed our default send size limit"),
        Ok(v) => v,
    };

    // Destructure the initiation
    let PalantirMessage::ClientInitiation { magic, name } = initiation else {
        // Create the error containing the unexpected packet errror
        let mut errs = vec![HandshakeError::UnexpectedPacket];

        // Tell the peer it sent an unexpected packet
        let res = framed.send(&PalantirMessage::UnexpectedPacket).await;

        // If there was another error, log it
        if let Err(e) = res {
            errs.push(e.into());
        }

        return Err(errs);
    };

    // Validate the magic
    if magic != "PALANTIR" {
        let mut errs = vec![HandshakeError::InvalidMagic];

        // Tell the peer it sent an invalid magic value
        let res = framed.send(&PalantirMessage::InvalidMagic).await;

        // If there was another error, log it
        if let Err(e) = res {
            errs.push(e.into());
        }

        return Err(errs);
    }


    // Check if the name exists. If it does, error.
    if instance.peers.read().expect("peers map not poisoned").contains_key(&name) {
        let mut errs = vec![HandshakeError::NameTaken];

        // Tell the peer its name is taken
        let res = framed.send(&PalantirMessage::NameTaken).await;

        // If there was another error, log it
        if let Err(e) = res {
            errs.push(e.into());
        }

        return Err(errs);
    }


    // # Step 2
    // Server sends response to client

    // Create the response
    let server_response = PalantirMessage::<V>::ClientInitiation {
        magic: "PALANTIR".to_string(),
        name: instance.name.clone(), // This instance's name
    };

    // Send the initiation
    framed.send(&server_response).await
        .map_err(|e| vec![e.into()])?;


    // # Step 3
    // Run validation
    instance.validator.handshake(&mut framed, state, &name).await?;

    // # Step 4
    // Await confirmation of finished handshake
    match framed.recv().await {
        Err(e @ FramedError::TransmissionError(_)) => {
            // Because this is the only error in the chain, we can just return a new vec.
            return Err(vec![e.into()]);
        }
        Err(e @ FramedError::InvalidEncoding { packet: _ }) => {
            // Create a vec with the error
            let mut errs: Vec<HandshakeError> = vec![e.into()];

            // Tell the peer it sent an invalid packet
            let res = framed.send(&PalantirMessage::MalformedData).await;

            // If there was another error, log it
            if let Err(e) = res {
                errs.push(e.into());
            }

            return Err(errs);
        }
        Err(_) => unreachable!("received packets can't exceed our default send size limit"),
        Ok(PalantirMessage::HandshakeCompleted) => (),
        Ok(_) => {
            // Create a vec with the error
            let mut errs: Vec<HandshakeError> = vec![HandshakeError::UnexpectedPacket];

            // Tell the peer it sent an unexpected packet
            let res = framed.send(&PalantirMessage::UnexpectedPacket).await;

            // If there was another error, log it
            if let Err(e) = res {
                errs.push(e.into());
            }

            return Err(errs);
        }
    };

    
    // # Step 5
    // Close the channel.
    framed.0.close().await;

    Ok(name)
}

/// # [`Side`]
/// Indicates which end of a connection the given method is executing on:
/// either the initiator or the acceptor.
/// (Client or server, but this distinction is important)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Side {
    Initiator,
    Acceptor,
}

/// # [`ActorID`]
/// A way to represent an actor over the network.
/// Can either be an actor's numerical ID, or a name for the actor.
#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum ActorID {
    ID(u64) = 0,
    Name(String) = 1,
}

/// # [`PalantirMessage`]
/// The message structure that palantir sends over the wire, serialized using [`postcard`]
#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum PalantirMessage<V: Validator> {
    /// # [`PalantirMessage::ClientInitiation`]
    /// Sent by the client to the server when it initiates a connection.
    /// Contains the client's name, and a magic string that should always be "PALANTIR"
    ClientInitiation { magic: String, name: String } = 0,
    /// # [`PalantirMessage::ServerResponse`]
    /// Send by the server to the client in response to a [`PalantirMessage::ClientInitiation`].
    /// Contains the server's name and the magic string ("PALANTIR").
    ServerResponse { magic: String, name: String } = 1,

    /// # [`PalantirMessage::ValidatorPacket`]
    /// Arbitrary packet exchanged by the validator after [`PalantirMessage::ServerResponse`]
    /// but before [`PalantirMessage::HandshakeCompleted`].
    ValidatorPacket(V::Packet) = 2,

    /// # [`PalantirMessage::HandshakeCompleted`]
    /// Send by the client to the server to indicate that the handshake was completed.
    HandshakeCompleted = 3,

    /// # [`PalantirMessage::Request`]
    /// Indicates a request with the given ID and arbitrary data.
    Request {
        /// The request's id.
        id: u32,
        /// The request's data
        data: Vec<u8>,
    } = 4,

    /// # [`PalantirMessage::Response`]
    /// Contains the response data to a request with the given ID.
    Response {
        /// The ID of the request this response is to
        id: u32,
        /// The response data
        data: Vec<u8>,
    } = 5,

    /// # [`PalantirMessage::NameTaken`]
    /// Sent by either peer to the other during the handshake
    /// to indicate that the given name was taken.
    /// The connection is then terminated.
    NameTaken,

    /// # [`PalantirMessage::InvalidMagic`]
    /// Sent by either peer to the other during the handshake
    /// to indicate that the given magic value was invalid.
    /// The connection is then terminated.
    InvalidMagic,

    /// # [`PalantirMessage::UnexpectedPacket`]
    /// Sent by either peer to the other to indicate that
    /// an unexpected packet was received.
    /// The connection is then terminated.
    UnexpectedPacket,

    /// # [`PalantirMessage::MalformedData`]
    /// Sent by either peer to the other to indicate
    /// that the data sent was malformed and unable
    /// to be deserialized.
    MalformedData,
}
