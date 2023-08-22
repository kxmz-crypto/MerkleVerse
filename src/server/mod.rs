mod mverse;

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
    connection_string: String,
    prefix: Index,
    length: u32,
    superior: Option<ServerCluster>,
    parallel: Option<ServerCluster>,
}

impl Default for Index{
    fn default() -> Self {
        Self {
            index: vec![],
            length: 0,
        }
    }
}
//TODO: add BLS signatures https://docs.rs/bls-signatures/0.14.0/bls_signatures/