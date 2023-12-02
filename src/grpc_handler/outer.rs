use crate::grpc_handler::inner::{mversegrpc};
use crate::grpc_handler::outer::mverseouter::{
    ClientTransactionRequest, Empty, PeerCommitRequest, PeerPrepareRequest, PeerTransactionRequest,
};
use crate::server;
use anyhow::{anyhow, Result};
pub use mversegrpc::{
    GetMerkleRootRequest, GetMerkleRootResponse, LookUpHistoryResponse, LookUpLatestRequest,
    LookUpLatestResponse, LookupHistoryRequest, TransactionRequest,
};
pub use mverseouter::{
    merkle_verse_client::MerkleVerseClient,
    merkle_verse_server::{MerkleVerse, MerkleVerseServer},
    transaction_response::TransactionResult,
    ServerInformationResponse, TransactionResponse,
};
use tonic::{IntoRequest, Request, Response, Status};

pub mod mverseouter {
    tonic::include_proto!("mverseouter");
}

fn err_transform(e: anyhow::Error) -> Status {
    Status::internal(e.to_string())
}

#[tonic::async_trait]
impl MerkleVerse for server::MerkleVerseServer {
    async fn get_server_information(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ServerInformationResponse>, Status> {
        Ok(Response::new(ServerInformationResponse {
            server_name: "Outer Merkle Verse Server".into(),
            server_id: self.id.0.clone(),
        }))
    }

    async fn look_up_history(
        &self,
        request: Request<LookupHistoryRequest>,
    ) -> Result<Response<LookUpHistoryResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req: mversegrpc::LookupHistoryRequest = request.into_inner();
        let res: LookUpHistoryResponse = inn_client
            .look_up_history(inn_req.into_request())
            .await?
            .into_inner();

        Ok(Response::new(res))
    }

    async fn get_current_root(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetMerkleRootResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req = mversegrpc::Empty {};
        let res: GetMerkleRootResponse = inn_client
            .get_current_root(inn_req.into_request())
            .await?
            .into_inner();
        let rsp = GetMerkleRootResponse { head: res.head };
        Ok(Response::new(rsp))
    }

    async fn get_root(
        &self,
        request: Request<GetMerkleRootRequest>,
    ) -> Result<Response<GetMerkleRootResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req: GetMerkleRootRequest = request.into_inner();
        let res: GetMerkleRootResponse = inn_client
            .get_root(inn_req.into_request())
            .await?
            .into_inner();
        let rsp = GetMerkleRootResponse { head: res.head };
        Ok(Response::new(rsp))
    }

    async fn look_up_latest(
        &self,
        request: Request<LookUpLatestRequest>,
    ) -> Result<Response<LookUpLatestResponse>, Status> {
        let mut inn_client = self.get_inner_client().await?;
        let inn_req: LookUpLatestRequest = request.into_inner();
        let res = inn_client
            .look_up_latest(inn_req.into_request())
            .await?
            .into_inner();
        Ok(Response::new(res))
    }

    async fn client_transaction(
        &self,
        request: Request<ClientTransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let inn_req = request.into_inner();
        let res = self
            .receive_client_transaction(inn_req)
            .await
            .map_err(err_transform)
            .map(|res| TransactionResponse {
                status: match res {
                    None => TransactionResult::Ok,
                    Some(_) => TransactionResult::Duplicate,
                }
                .into(),
            })?;
        Ok(Response::new(res))
    }

    async fn peer_transaction(
        &self,
        request: Request<PeerTransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let inn_req = request.into_inner();
        let res = self
            .receive_peer_transaction(inn_req)
            .await
            .map_err(err_transform)
            .map(|res| TransactionResponse {
                status: match res {
                    None => TransactionResult::Ok,
                    Some(_) => TransactionResult::Duplicate,
                }
                .into(),
            })?;
        Ok(Response::new(res))
    }

    async fn peer_prepare(
        &self,
        request: Request<PeerPrepareRequest>,
    ) -> Result<Response<Empty>, Status> {
        let inn_req = request.into_inner();
        self.receive_prepare(
            inn_req
                .epoch
                .ok_or(anyhow!("An epoch number must be provided!"))
                .map_err(err_transform)?
                .epoch,
            inn_req
                .peer_identity
                .ok_or(anyhow!("A peer identity must be provided!"))
                .map_err(err_transform)?
                .server_id
                .into(),
        )
        .await
        .map_err(err_transform)?;
        Ok(Response::new(Empty {}))
    }

    async fn peer_commit(
        &self,
        request: Request<PeerCommitRequest>,
    ) -> Result<Response<Empty>, Status> {
        let inn_req = request.into_inner();
        self.receive_signatures(
            inn_req
                .epoch
                .ok_or(anyhow!("An epoch number must be provided!"))
                .map_err(err_transform)?
                .epoch,
            &inn_req.head,
            &inn_req.signature,
        )
        .await
        .map_err(err_transform)?;
        Ok(Response::new(Empty {}))
    }
}
