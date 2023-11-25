use std::collections::HashMap;
use crate::server::{MerkleVerseServer, ServerId};
use anyhow::{anyhow, Result};
use tonic::IntoRequest;
use crate::grpc_handler::inner::mversegrpc::Epoch;
use crate::grpc_handler::outer::mverseouter::{ClientTransactionRequest, PeerPrepareRequest, PeerTransactionRequest, ServerIdentity};


#[derive(Debug)]
pub enum Transaction{
    Client(ClientTransactionRequest),
    Peer(PeerTransactionRequest)
}

#[derive(Debug)]
pub struct MultiSig{
    pub epoch: u32,
    pub root: Vec<u8>,
    pub aggregate: Vec<u8>,
    pub signatures: HashMap<ServerId, Vec<u8>>
}

#[derive(Debug, Default)]
pub enum RunState{
    Prepare(u64),
    #[default]
    Normal
}

#[derive(Debug)]
pub struct MerkleVerseServerState {
    pub current_root: Vec<u8>,
    pub current_epoch: u64,
    pub pending_transactions: HashMap<u64, Vec<Transaction>>, // Remember to remove transactions from this list when they are added to the tree
    pub multi_sigs: HashMap<u64, MultiSig>,
    pub run_state: RunState,
    pub peer_states: HashMap<ServerId, MerkleVerseServerState>
}

impl MerkleVerseServer {
    pub async fn broadcast_prepare(&self) -> Result<()> {
        let cur_epoch = {
            let mut serv_state = self.state.lock().unwrap();
            if matches!(serv_state.run_state, RunState::Prepare(_)){
                return Err(anyhow!("Server is already in prepare state"));
            }
            serv_state.run_state = RunState::Prepare(serv_state.current_epoch);
            serv_state.current_epoch
        };
        if let Some(servers) = &self.parallel {
            let mut futures = vec![];
            for srv in &servers.servers {
                let srv = srv.clone();
                let mut client = srv.get_client().await?;
                let fut = async move {
                    client.peer_prepare(PeerPrepareRequest{
                        epoch: Some(Epoch{
                            epoch: cur_epoch as u64
                        }),
                        peer_identity: Some(ServerIdentity{
                            server_id: self.id.0.clone()
                        })
                    }.into_request()).await
                };
                futures.push(fut);
            }
            futures::future::join_all(futures).await;
        }
        Ok(())
    }

    pub async fn receive_prepare(&self, epoch: u64, server_id: &ServerId) -> Result<()> {
        let mut serv_state = self.state.lock().unwrap();
        let mut peer_state = serv_state.peer_states.get_mut(server_id).unwrap();
        peer_state.run_state = RunState::Prepare(epoch);
        if !matches!(serv_state.run_state, RunState::Prepare(_)){
            self.broadcast_prepare().await?;
        }
        Ok(())
    }

    pub async fn sign_and_broadcast(&self) -> Result<()> {
        /// Signs the current tree root with BLS, and broadcasts it to the parallel servers.
        todo!()
    }

    pub async fn receive_signatures(&self) -> Result<()> {
        /// receives the signatures from the parallel servers regarding an epoch. If the contents match
        /// the current root, then the state of multisignature is updated.
        todo!()
    }

    pub async fn watch_trigger_prepare(&self) -> Result<()> {
        /// watches the current epoch, and triggers the prepare phase when the epoch interval is reached.
        /// also might trigger prepare if a certain number of transactions are received.
        todo!()
    }

    pub async fn trigger_sign_and_broadcast(&self) -> Result<()> {
        /// triggers the sign and broadcast phase when the prepare phase is finished,
        /// and when enough transactions are received.
        /// Note: might need to base this on the server configuration
        todo!()
    }
}
