use crate::server::PeerServer;
use std::sync::Arc;

struct Prepare {
    epoch: u32,
    root: Vec<u8>,
    source: Arc<PeerServer>,
}

struct EpochSignature {
    root: Vec<u8>,
    signature: Vec<u8>,
    source: Arc<PeerServer>,
    epoch: u32,
}

struct PeerTransaction {
    epoch: u32,
    root: Vec<u8>,
    transaction: Vec<u8>,
    source: Arc<PeerServer>,
}
