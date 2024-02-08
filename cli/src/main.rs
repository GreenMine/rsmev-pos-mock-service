use pos_mock::PosMock;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    rsmev::serve(listener, PosMock::new()).await.unwrap()
}
