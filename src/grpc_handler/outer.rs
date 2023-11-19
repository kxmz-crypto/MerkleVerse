use tonic::{IntoRequest, Request, Response, Status};
pub use mverseouter::{ServerInformationResponse, merkle_verse_server::{MerkleVerseServer, MerkleVerse}};
pub use mverseouter::merkle_verse_client::MerkleVerseClient;
pub use mversegrpc::{TransactionRequest, TransactionResponse, LookupHistoryRequest, LookUpHistoryResponse, GetMerkleRootRequest, GetMerkleRootResponse, LookUpLatestRequest, LookUpLatestResponse};
use crate::grpc_handler::outer::mverseouter::{Empty};
use crate::grpc_handler::inner::{MerkleProviderClient, mversegrpc};
use crate::server;
use anyhow::Result;

pub mod mverseouter {
    tonic::include_proto!("mverseouter");
}


#[tonic::async_trait]
impl MerkleVerse for server::MerkleVerseServer {
    async fn get_server_information(&self, _request: Request<Empty>) -> Result<Response<ServerInformationResponse>, Status> {
         Ok(Response::new(ServerInformationResponse{
                 server_name: "Outer Merkle Verse Server".into(),
             }))
    }

    async fn look_up_history(&self, request: Request<LookupHistoryRequest>) -> Result<Response<LookUpHistoryResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req : mversegrpc::LookupHistoryRequest= request.into_inner().into();
        let res: LookUpHistoryResponse = inn_client.look_up_history(
            inn_req.into_request()
        ).await?.into_inner().into();

        Ok(Response::new(res))
    }

    async fn transaction(&self, request: Request<TransactionRequest>) -> Result<Response<TransactionResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req : mversegrpc::TransactionRequest = request.into_inner().into();
        let res: TransactionResponse = inn_client.transaction(inn_req.into_request()).await?.into_inner().into();
        Ok(Response::new(res))
    }

    async fn get_current_root(&self, _request: Request<Empty>) -> Result<Response<GetMerkleRootResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req = mversegrpc::Empty{};
        let res : GetMerkleRootResponse = inn_client
            .get_current_root(inn_req.into_request())
            .await?
            .into_inner()
            .into();
        let rsp = GetMerkleRootResponse{
            head: res.head
        };
        Ok(Response::new(rsp))
    }

    async fn get_root(&self, request: Request<GetMerkleRootRequest>) -> Result<Response<GetMerkleRootResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req : mversegrpc::GetMerkleRootRequest = request.into_inner().into();
        let res : GetMerkleRootResponse = inn_client.get_root(inn_req.into_request())
            .await?
            .into_inner()
            .into();
        let rsp = GetMerkleRootResponse{
            head: res.head
        };
        Ok(Response::new(rsp))
    }

    async fn look_up_latest(&self, request: Request<LookUpLatestRequest>) -> Result<Response<LookUpLatestResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req : mversegrpc::LookUpLatestRequest = request.into_inner().into();
        let res = inn_client.look_up_latest(inn_req.into_request())
            .await?
            .into_inner()
            .into();
        Ok(Response::new(res))
    }
}