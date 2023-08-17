use tonic::{Request, Response, Status};
pub use mverseouter::{ServerInformationResponse, merkle_verse_server::{MerkleVerseServer, MerkleVerse} };
use crate::grpc_handler::outer::mverseouter::{Empty, GetMerkleRootRequest, GetMerkleRootResponse, LookupHistoryRequest, LookUpHistoryResponse, LookUpLatestRequest, LookUpLatestResponse, TransactionRequest, TransactionResponse};

pub mod mverseouter {
    tonic::include_proto!("mverseouter");
}

#[derive(Debug, Default)]
pub struct OuterMerkleVerseServer {}

#[tonic::async_trait]
impl MerkleVerse for OuterMerkleVerseServer {
    async fn get_server_information(&self, request: Request<Empty>) -> Result<Response<ServerInformationResponse>, Status> {
         Ok(Response::new((
             ServerInformationResponse{
                 server_name: "Outer Merkle Verse Server".into(),
             }
         )))
    }

    async fn look_up_history(&self, request: Request<LookupHistoryRequest>) -> Result<Response<LookUpHistoryResponse>, Status> {
        todo!()
    }

    async fn transaction(&self, request: Request<TransactionRequest>) -> Result<Response<TransactionResponse>, Status> {
        todo!()
    }

    async fn get_current_root(&self, request: Request<Empty>) -> Result<Response<GetMerkleRootResponse>, Status> {
        todo!()
    }

    async fn get_root(&self, request: Request<GetMerkleRootRequest>) -> Result<Response<GetMerkleRootResponse>, Status> {
        todo!()
    }

    async fn look_up_latest(&self, request: Request<LookUpLatestRequest>) -> Result<Response<LookUpLatestResponse>, Status> {
        todo!()
    }
}