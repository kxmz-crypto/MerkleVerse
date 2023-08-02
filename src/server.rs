struct PeerServer {
//     TODO implement this type
}

struct ServerCluster{
    servers: Vec<PeerServer>,
}

struct Index{
    index: Vec<u8>,
    length: u32
}

/// the `MerkleVerseServer` struct records the current server's location within the
/// Merkle Verse system.
pub struct MerkleVerseServer{
    index: Index,
    prefix: Index,
    superior: Option<ServerCluster>,
    parallel: Option<ServerCluster>,
}