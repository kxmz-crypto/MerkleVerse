mod messages;
mod mverse;
mod synchronization;
mod transactions;

use crate::utils;

use crate::server::mverse::MServerPointer;
use anyhow::Result;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use bls_signatures::Serialize;
use crate::server::synchronization::MerkleVerseServerState;

#[derive(Debug, Clone, Eq, PartialEq)]
struct PublicKey{
    raw: Vec<u8>,
    pub bls: bls_signatures::PublicKey
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PrivateKey{
    raw: Vec<u8>,
    pub bls: bls_signatures::PrivateKey
}

impl TryFrom<Vec<u8>> for PrivateKey {
    type Error = anyhow::Error;
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            bls: bls_signatures::PrivateKey::from_bytes(&value)?,
            raw: value,
        })
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = anyhow::Error;
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            bls: bls_signatures::PublicKey::from_bytes(&value)?,
            raw: value,
        })
    }
}

struct Signature {
    signature: Vec<u8>,
}

struct EpochInfo {
    head: Vec<u8>,
    signatures: Vec<Signature>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PeerServer {
    connection_string: String,
    id: ServerId,
    prefix: Index,
    length: u32,
    epoch_interval: u32,
    public_key: PublicKey,
}

#[derive(Debug, Clone)]
struct ServerCluster {
    prefix: Option<Index>,
    servers: Vec<PeerServer>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Index {
    index: Vec<u8>,
    length: u32,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ServerId(pub String);

impl From<String> for ServerId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// the `MerkleVerseServer` struct records the current server's location within the
/// Merkle Verse system.
#[derive(Debug, Clone)]
pub struct MerkleVerseServer {
    inner_dst: String,
    pub connection_string: String,
    pub id: ServerId,
    prefix: Index,
    length: u32,
    superior: Option<ServerCluster>,
    parallel: Option<ServerCluster>,
    epoch_interval: u32,
    private_key: PrivateKey,
    public_key: PublicKey,
    state: Arc<Mutex<MerkleVerseServerState>>
}


impl Default for Index {
    fn default() -> Self {
        Self {
            index: vec![],
            length: 0,
        }
    }
}

impl Index {
    pub fn from_b64(b64: &str, length: u32) -> Result<Self> {
        let index = utils::b64_to_loc(b64, usize::try_from(length)?)?;
        Ok(Self { index, length })
    }

    pub fn to_binstring(&self) -> Result<String> {
        Ok(utils::binary_string(
            &self.index,
            usize::try_from(self.length)?,
        ))
    }
}

impl From<Vec<u8>> for Index{
    fn from(value: Vec<u8>) -> Self {
        Self{
            length: value.len() as u32,
            index: value,
        }
    }
}

impl From<Vec<MServerPointer>> for ServerCluster {
    fn from(servers: Vec<MServerPointer>) -> Self {
        Self {
            prefix: None,
            servers: servers
                .iter()
                .map(|x| {
                    let srv = x.borrow();
                    PeerServer {
                        connection_string: format!("http://{}", srv.connection_string.clone()),
                        prefix: srv.prefix.clone(),
                        length: srv.length,
                        epoch_interval: srv.epoch_interval,
                        id: srv.id.clone(),
                        public_key: srv.public_key.clone(),
                    }
                })
                .collect(),
        }
    }
}
