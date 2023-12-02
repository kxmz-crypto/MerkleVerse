mod messages;
mod mverse;
mod synchronization;
mod transactions;
mod validation;

use crate::utils;
use std::collections::HashMap;

use crate::server::mverse::MServerPointer;
use crate::server::synchronization::MerkleVerseServerState;
use anyhow::Result;

use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use validation::{PrivateKey, PublicKey};

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
    servers: HashMap<ServerId, PeerServer>,
}

impl ServerCluster {
    fn get_server(&self, id: &ServerId) -> Option<&PeerServer> {
        self.servers.get(id)
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[derive(Default)]
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
    state: Arc<Mutex<MerkleVerseServerState>>, // TODO: consider using a RwLock instead
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

impl From<Vec<u8>> for Index {
    fn from(value: Vec<u8>) -> Self {
        Self {
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
                    (
                        srv.id.clone(),
                        PeerServer {
                            connection_string: format!("http://{}", srv.connection_string.clone()),
                            prefix: srv.prefix.clone(),
                            length: srv.length,
                            epoch_interval: srv.epoch_interval,
                            id: srv.id.clone(),
                            public_key: srv.public_key.clone(),
                        },
                    )
                })
                .collect(),
        }
    }
}
