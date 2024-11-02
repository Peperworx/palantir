use std::sync::Arc;

use fluxion::{actor, message, Fluxion, Identifier};
use palantir::Palantir;

#[actor]
struct TestActor;

#[message]
struct TestMessage(pub String);

#[tokio::main]
async fn main() {

    // Create palantir instance that will listen on port 8080
    let delegate = Arc::new(Palantir::new(8080));

    // Create fluxion instance with the palantir delegate
    let system = Fluxion::new("host", delegate.clone());

    // Register an event for a new peer connecting to us
    let s = system.clone();
    delegate.on_new_connection(|new_id| async {
        let a = s.get(Identifier::Foreign(0, new_id)).await.unwrap();
        a.send(TestMessage).await;
    });

    // Spawn a new actor
    let actor_id = system.add(TestActor).await.unwrap();

    // Add a peer named "other"
    delegate.add_peer("other", "https://localhost:8080").await;

    

    // Begin running the delegate's main loop.
    let jh = tokio::spawn(delegate.run(system.clone()));

    // Wait for the delegate to exit
    jh.await;
}