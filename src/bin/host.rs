use std::{net::{SocketAddr, SocketAddrV4}, time::Duration};

use palantir::layers::{web_transport::host::WTHost, Namespace, Peer};
use wtransport::{Certificate, ServerConfig};

#[tokio::main]
pub async fn main() {

    // Create the WTHost
    let server = WTHost::<String>::start(
        ServerConfig::builder()
            .with_bind_default(4433)
            .with_certificate(
                Certificate::self_signed(vec!["test_host"])
            )
            .build()
    ).await.unwrap();

    println!("started server");

    // Wait for a client
    let client = loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        
    };
    /* 
    // Wait for a new namespace
    let mut ns = client.wait_namespace().await.unwrap();

    // print its id
    println!("{:?}", ns.get_id());

    // Receive some data
    let recvd = ns.recv().await.unwrap();

    println!("received data: {}", recvd);
    */
}