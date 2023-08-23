use crate::grpc_handler::outer::{MerkleVerseServer};
use crate::config::ServersConfig;
use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;

use tonic::transport::Server;

mod grpc_handler;
mod server;
mod bridge;
mod config;
mod utils;

#[derive(Parser, Debug)]
#[clap(name = "MerkleVerse", version = "0.1.0", author = "JettChenT")]
struct Args{
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let cfig = ServersConfig::with_path(args.config.as_path())?;
    let server = server::MerkleVerseServer::from_cluster_config(
        cfig
    ).await?;
    let conn = server.connection_string.clone();

    let server_cl = server.clone();
    let epoch_loop = tokio::spawn(async move {
        &server_cl.epoch_loop().await;
    });

    eprintln!("Server starting at {}", conn);
    Server::builder()
        .add_service(MerkleVerseServer::new(server))
        .serve(conn.parse()?)
        .await?;

    tokio::try_join!(epoch_loop)?;
    Ok(())
}
