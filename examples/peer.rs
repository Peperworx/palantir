use std::env::args;

use palantir::peer::Peer;



#[tokio::main]
pub async fn main() {

    // Create a new peer
    let peer = Peer::new(args().nth(1).unwrap().parse().unwrap());

    // Run this peer
    let mut channels = peer.run().await;

    tokio::spawn(async move {
        loop {
            let next = channels.recv().await;

            if let Some(next) = next {
                println!("New channel!");
            }
        }
    });

    // Add another peer
    let name = peer.add(args().nth(2).unwrap()).await.unwrap();

    println!("Added peer {name}");

    // Open a channnel to that peer
    let channel = peer.open_channel(&name).await.unwrap();
    
    println!("Opened a channel to a peer!");

}