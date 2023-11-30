use ecdsa::{SigningKey, VerifyingKey};
use palantir::transport::wtransport::DirectPeer;
use rand_core::OsRng;
#[tokio::main]
async fn main() {
    let sk = SigningKey::random(&mut OsRng);
    let vk = VerifyingKey::from(&sk);

    let wt = DirectPeer::new("0.0.0.0:4433".parse().unwrap(), (sk, vk)).unwrap();

    wt.clone().run().await;
}