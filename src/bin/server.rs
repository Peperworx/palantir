
use elliptic_curve::SecretKey;
use p384::NistP384;
use palantir::transport::wtransport::DirectPeer;
use rand_core::OsRng;
#[tokio::main]
async fn main() {
    let sk = SecretKey::<NistP384>::random(&mut OsRng);
    

    let wt = DirectPeer::new("0.0.0.0:4433".parse().unwrap(), sk).unwrap();

    wt.clone().run().await;
}