mod mverse;
use crate::utils;

use anyhow::Result;
use std::convert::TryFrom;
use crate::server::mverse::MServerPointer;

struct Signature{
    signature: Vec<u8>
}

struct EpochInfo{
    head: Vec<u8>,
    signatures: Vec<Signature>,
}

#[derive(Debug, Clone)]
pub struct PeerServer {
    connection_string: String,
    prefix: Index,
    length: u32,
    epoch_interval: u32,
}

#[derive(Debug, Clone)]
struct ServerCluster{
    prefix: Option<Index>,
    servers: Vec<PeerServer>,
}

#[derive(Debug, Clone)]
struct Index{
    index: Vec<u8>,
    length: u32
}

/// the `MerkleVerseServer` struct records the current server's location within the
/// Merkle Verse system.
#[derive(Debug, Clone)]
pub struct MerkleVerseServer{
    inner_dst: String,
    pub connection_string: String,
    prefix: Index,
    length: u32,
    superior: Option<ServerCluster>,
    parallel: Option<ServerCluster>,
    epoch_interval: u32
}

impl Default for Index{
    fn default() -> Self {
        Self {
            index: vec![],
            length: 0,
        }
    }
}

impl Index{
    pub fn from_b64(b64:&str, length: u32) -> Result<Self>{
        let index = utils::b64_to_loc(b64, usize::try_from(length)?)?;
        Ok(Self{
            index,
            length,
        })
    }

    pub fn to_binstring(&self) -> Result<String> {
        Ok(utils::binary_string(&self.index, usize::try_from(self.length)?))
    }
}

impl From<Vec<MServerPointer>> for ServerCluster{
    fn from(servers: Vec<MServerPointer>) -> Self {
        Self{
            prefix: None,
            servers: servers.iter().map(|x| {
                let srv = x.borrow();
                PeerServer {
                    connection_string: format!("http://{}",srv.connection_string.clone()),
                    prefix: srv.prefix.clone(),
                    length: srv.length,
                    epoch_interval: srv.epoch_interval,
                }
            }).collect(),
        }
    }
}

//TODO: add BLS signatures https://docs.rs/bls-signatures/0.14.0/bls_signatures/