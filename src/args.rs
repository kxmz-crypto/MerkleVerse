use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "MerkleVerse", version = "0.1.0", author = "JettChenT")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Server(ServerArgs),
    GenPeers(GenPeerArgs),
}

#[derive(Parser, Debug)]
pub struct ServerArgs {
    #[arg(short, long)]
    pub config: PathBuf,
}

#[derive(Parser, Debug)]
pub struct GenPeerArgs {
    #[arg(short, long)]
    pub num: u32,
    #[arg(short, long)]
    pub dest: PathBuf,
}