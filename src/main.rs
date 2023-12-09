use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use ::config::ConfigBuilder;
use crate::config::ServersConfig;
use crate::grpc_handler::outer::{MerkleVerseServer};
use anyhow::Result;
use clap::Parser;
use futures::{FutureExt, TryFutureExt};
use tonic::codegen::Body;

use args::{Args, Commands, ServerArgs};
use tonic::transport::Server;
use tonic_reflection::pb::FILE_DESCRIPTOR_SET;
use tower_http::trace::TraceLayer;
use tower::{BoxError, Service, ServiceExt, steer::Steer};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::registry::Data;
use tracing_subscriber::util::SubscriberInitExt;
use crate::args::GenPeerArgs;
use crate::metaconfig::MetaConfig;

mod args;
mod bridge;
mod config;
mod grpc_handler;
mod metaconfig;
mod server;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    initialize_tracing()?;
    let args = Args::parse();
    match args.command {
        Commands::Server(s) => srv(s).await?,
        Commands::GenPeers(g) => gen_configs(g)?
    }
    Ok(())
}

async fn srv(args: ServerArgs) -> Result<()> {
    let cfig = ServersConfig::with_path(args.config.as_path())?;
    let server = server::MerkleVerseServer::from_cluster_config(cfig).await?;
    let conn = server.connection_string.clone();

    let server_cl = server.clone();
    let routine_loop = tokio::spawn(async move {
        let res = server_cl.routine().await;
        match res {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("When running routine tasks: {}", e);
            }
        }
    });

    tracing::info!("Server starting at {}", conn);

    let _grpc_service = Server::builder()
        .trace_fn(|_| tracing::info_span!("MerkleVerse Server"))
        .add_service(MerkleVerseServer::new(server))
        .serve(conn.parse()?)
        .await?;

    tokio::try_join!(routine_loop)?;
    Ok(())
}

fn gen_configs(args: GenPeerArgs) -> Result<()> {
    let config = MetaConfig::with_path(args.src)?;
    let srvs = config.to_serv_configs()?;
    match args.to {
        None => {
            for srv in srvs {
                println!("{}\n", srv.to_string())
            }
        }
        Some(path) => {
            for srv in srvs{
                fs::write(path.join(format!("{}.toml", srv.server.server_config.id)), srv.to_string())?;
            }
        }
    }
    Ok(())
}

fn initialize_tracing() -> Result<()>{
    tracing_subscriber::fmt::fmt()
        .with_line_number(true)
        .with_file(true)
        .with_thread_ids(true)
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();
    Ok(())
}