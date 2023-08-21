use crate::grpc_handler::outer::{MerkleVerseServer, OuterMerkleVerseServer};

use tonic::transport::Server;

mod grpc_handler;
mod server;
mod bridge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:1319".parse().unwrap();
    let server = OuterMerkleVerseServer::default();
    Server::builder()
        .add_service(MerkleVerseServer::new(server))
        .serve(addr)
        .await?;
    Ok(())
}
