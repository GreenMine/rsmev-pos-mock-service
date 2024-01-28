mod pos;
mod rsmev;
mod service;
use pos::Pos;
pub use service::Service;

use tokio::net::TcpListener;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    rsmev::serve(listener, Pos::new()).await.unwrap()
}
