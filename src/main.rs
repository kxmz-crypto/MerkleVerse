use crate::config::ServersConfig;
use crate::grpc_handler::outer::MerkleVerseServer;
use anyhow::Result;
use clap::Parser;

use args::{Args, Commands, ServerArgs};
use tonic::transport::Server;

mod args;
mod bridge;
mod config;
mod grpc_handler;
mod server;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Server(s) => srv(s).await?,
        Commands::GenPeers(_) => {}
    }
    Ok(())
}

async fn srv(args: ServerArgs) -> Result<()> {
    let cfig = ServersConfig::with_path(args.config.as_path())?;
    let server = server::MerkleVerseServer::from_cluster_config(cfig).await?;
    let conn = server.connection_string.clone();

    let server_cl = server.clone();
    let prep_loop = tokio::spawn(async move {
        let res = server_cl.watch_trigger_prepare().await;
        match res {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error in trigger prepare loop: {}", e);
            }
        }
    });

    eprintln!("Server starting at {}", conn);
    Server::builder()
        .add_service(MerkleVerseServer::new(server))
        .serve(conn.parse()?)
        .await?;

    tokio::try_join!(prep_loop)?;
    Ok(())
}
