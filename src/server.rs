struct PeerServer {
//     TODO implement this type
}

struct ServerCluster{
    servers: Vec<PeerServer>,
}

pub struct MerkleVerseServer{
    superior: Option<ServerCluster>,
    parallel: Option<ServerCluster>,
}