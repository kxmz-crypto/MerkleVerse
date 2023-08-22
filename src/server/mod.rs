mod mverse;
use crate::utils;

use anyhow::Result;
use std::convert::TryFrom;

struct Signature{
    signature: Vec<u8>
}

struct EpochInfo{
    head: Vec<u8>,
    signatures: Vec<Signature>,
}

#[derive(Debug)]
struct PeerServer {
//     TODO implement this type
}

#[derive(Debug)]
struct ServerCluster{
    prefix: Index,
    servers: Vec<PeerServer>,
}

#[derive(Debug)]
struct Index{
    index: Vec<u8>,
    length: u32
}

/// the `MerkleVerseServer` struct records the current server's location within the
/// Merkle Verse system.
#[derive(Debug)]
pub struct MerkleVerseServer{
    inner_dst: String,
    connection_string: String,
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
}
//TODO: add BLS signatures https://docs.rs/bls-signatures/0.14.0/bls_signatures/