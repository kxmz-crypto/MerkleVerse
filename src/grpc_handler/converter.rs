use crate::grpc_handler::{inner::mversegrpc, outer::mverseouter};

impl From<mverseouter::Hash> for mversegrpc::Hash {
    fn from(value: mverseouter::Hash) -> Self {
        mversegrpc::Hash{
            hash: value.hash
        }
    }
}

impl From<mversegrpc::Hash> for mverseouter::Hash {
    fn from(value: mversegrpc::Hash) -> Self {
        mverseouter::Hash{
            hash: value.hash
        }
    }
}

impl From<mversegrpc::Epoch> for mverseouter::Epoch {
    fn from(value: mversegrpc::Epoch) -> Self {
        mverseouter::Epoch{
            epoch: value.epoch
        }
    }
}

impl From<mverseouter::Epoch> for mversegrpc::Epoch {
    fn from(value: mverseouter::Epoch) -> Self {
        mversegrpc::Epoch{
            epoch: value.epoch
        }
    }
}

impl From<mverseouter::MerklePath> for mversegrpc::MerklePath{
    fn from(value: mverseouter::MerklePath) -> Self {
        return mversegrpc::MerklePath{
            nodes: value.nodes.iter().map(
                |x|{
                    x.to_owned().into()
                }
            ).collect()
        }
    }
}

impl From<mversegrpc::MerkleProof> for mverseouter::MerkleProof {
    fn from(value: mversegrpc::MerkleProof) -> Self {
        mverseouter::MerkleProof{
            proof_type: value.proof_type,
            copath: value.copath.map(|cp| cp.into())
        }
    }
}

impl From<mversegrpc::MerklePath> for mverseouter::MerklePath{
    fn from(value: mversegrpc::MerklePath) -> Self {
        return mverseouter::MerklePath{
            nodes: value.nodes.iter().map(
                |x|{
                    x.to_owned().into()
                }
            ).collect()
        }
    }
}

impl From<mverseouter::MerkleProof> for mversegrpc::MerkleProof {
    fn from(value: mverseouter::MerkleProof) -> Self {
        mversegrpc::MerkleProof{
            proof_type: value.proof_type,
            copath: value.copath.map(|cp| cp.into())
        }
    }
}

impl From<mverseouter::LookUpLatestRequest> for mversegrpc::LookUpLatestRequest {
    fn from(value: mverseouter::LookUpLatestRequest) -> Self {
        mversegrpc::LookUpLatestRequest{
            key: value.key
        }
    }
}

impl From<mverseouter::LookupHistoryRequest> for mversegrpc::LookupHistoryRequest {
    fn from(value: mverseouter::LookupHistoryRequest) -> Self {
        mversegrpc::LookupHistoryRequest{
            key: value.key,
            n: value.n,
            lookup_type: value.lookup_type
        }
    }
}

impl From<mverseouter::GetMerkleRootRequest> for mversegrpc::GetMerkleRootRequest {
    fn from(value: mverseouter::GetMerkleRootRequest) -> Self {
        mversegrpc::GetMerkleRootRequest{
            epoch: value.epoch.map(|ep| ep.into())
        }
    }
}

impl From<mverseouter::TransactionRequest> for mversegrpc::TransactionRequest{
    fn from(value: mverseouter::TransactionRequest) -> Self {
        mversegrpc::TransactionRequest{
            value: value.value,
            key: value.key,
            origin: value.origin,
            transaction_type: value.transaction_type
        }
    }
}



impl From<mversegrpc::LookUpLatestResponse> for mverseouter::LookUpLatestResponse {
    fn from(value: mversegrpc::LookUpLatestResponse) -> Self {
        mverseouter::LookUpLatestResponse{
            value: value.value,
            proof: value.proof.map(|pf| pf.into()),
            head: value.head
        }
    }
}

impl From<mversegrpc::LookUpHistoryResponse> for mverseouter::LookUpHistoryResponse{
    fn from(value: mversegrpc::LookUpHistoryResponse) -> Self {
        return mverseouter::LookUpHistoryResponse{
            values: value.values,
            proof: value.proof.iter().map(|pf| pf.to_owned().into()).collect(),
            head: value.head
        }
    }
}

impl From<mversegrpc::GetMerkleRootResponse> for mverseouter::GetMerkleRootResponse {
    fn from(value: mversegrpc::GetMerkleRootResponse) -> Self {
        mverseouter::GetMerkleRootResponse{
            head: value.head,
        }
    }
}

impl From<mversegrpc::TransactionResponse> for mverseouter::TransactionResponse {
    fn from(value: mversegrpc::TransactionResponse) -> Self {
        mverseouter::TransactionResponse{
            head: value.head,
            status: value.status
        }
    }
}