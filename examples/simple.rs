use fluxion::{actor, message, Fluxion, Handler, Identifier};
use palantir::{backend::{Backend, Channel}, ActorID, Palantir};
use serde::{Deserialize, Serialize};


pub struct TestingBackend;

impl Backend for TestingBackend {
    type Channel = TestingChannel;

    async fn open_channel<M: fluxion::Message>(&self, actor: ActorID, system: &str, message_type: &'static str) -> Option<Self::Channel> {
        
        println!("Opening dummy channel for {:?}/{}", actor, system);
        Some(TestingChannel(actor, system.to_string()))
    }
}

pub struct TestingChannel(ActorID, String);

impl Channel for TestingChannel {
    async fn request(&self, data: Vec<u8>) -> Option<Vec<u8>> {
        println!("Dummy request: {:?}/{} sent: {:?}", self.0, self.1, data);
        Some(b"hello, world!".to_vec())
    }
}

#[actor]
struct TestActor;

impl Handler<TestMessage> for TestActor {
    async fn handle_message<D: fluxion::Delegate>(&self, message: TestMessage, context: &fluxion::ActorContext<D>) -> () {
        println!("test message won't be received");
    }
}
#[message]
#[derive(Serialize, Deserialize)]
struct TestMessage;

#[tokio::main]
async fn main() {

    let backend = TestingBackend;
    let delegate = Palantir::new("sys1".to_string(), backend);
    let system = Fluxion::new("sys1", delegate);

    // Open a test on another channel
    let mh = system.get::<TestActor, _>(Identifier::ForeignNamed("sys2", "testactor")).await.unwrap();

    //mh.send(TestMessage).await;

    system.shutdown().await;
}