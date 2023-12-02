use crate::grpc_handler::inner::mversegrpc::Epoch;
use crate::grpc_handler::outer::mverseouter::{
    ClientTransactionRequest, PeerCommitRequest, PeerPrepareRequest, PeerTransactionRequest,
    ServerIdentity,
};
use crate::grpc_handler::outer::TransactionRequest;
use crate::server::transactions::{TransactionPool};
use crate::server::{MerkleVerseServer, ServerId};
use anyhow::{anyhow, Result};
use bls_signatures::{aggregate, Serialize, Signature};
use std::collections::{HashMap};
use tonic::IntoRequest;

#[derive(Debug)]
pub struct MultiSig {
    pub epoch: u64,
    pub root: Vec<u8>,
    pub aggregate: Signature,
    pub signatures: HashMap<ServerId, Signature>,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum RunState {
    Prepare(u64),
    #[default]
    Normal,
}

#[derive(Debug, Default)]
pub struct MerkleVerseServerState {
    pub current_root: Vec<u8>,
    pub current_epoch: u64,
    multi_sigs: HashMap<u64, MultiSig>,
    run_state: RunState,
    peer_states: HashMap<ServerId, MerkleVerseServerState>,
    transaction_pool: TransactionPool,
}

impl MerkleVerseServer {
    fn server_identity(&self) -> ServerIdentity {
        ServerIdentity {
            server_id: self.id.0.clone(),
        }
    }

    pub async fn broadcast_prepare(&self) -> Result<()> {
        let cur_epoch = {
            let mut serv_state = self.state.lock().unwrap();
            if matches!(serv_state.run_state, RunState::Prepare(_)) {
                return Err(anyhow!("Server is already in prepare state"));
            }
            serv_state.run_state = RunState::Prepare(serv_state.current_epoch);
            serv_state.current_epoch
        };
        if let Some(servers) = &self.parallel {
            let mut futures = vec![];
            for (_, srv) in &servers.servers {
                let srv = srv.clone();
                let mut client = srv.get_client().await?;
                let fut = async move {
                    client
                        .peer_prepare(
                            PeerPrepareRequest {
                                epoch: Some(Epoch {
                                    epoch: cur_epoch as u64,
                                }),
                                peer_identity: Some(ServerIdentity {
                                    server_id: self.id.0.clone(),
                                }),
                            }
                            .into_request(),
                        )
                        .await
                };
                futures.push(fut);
            }
            futures::future::join_all(futures).await;
        }
        Ok(())
    }

    pub async fn receive_prepare(&self, epoch: u64, server_id: ServerId) -> Result<()> {
        let serv_runstate = {
            let mut serv_state = self.state.lock().unwrap();
            let peer_state = serv_state.peer_states.get_mut(&server_id).unwrap();
            peer_state.run_state = RunState::Prepare(epoch);
            serv_state.run_state
        };

        if !matches!(serv_runstate, RunState::Prepare(_)) {
            self.broadcast_prepare().await?;
        }
        Ok(())
    }

    pub async fn sign_and_broadcast(&self) -> Result<()> {
        /// Signs the current tree root with BLS, and broadcasts it to the parallel servers.
        let (epoch, sig, head) = {
            let mut serv_state = self.state.lock().unwrap();
            let sig = self.private_key.bls.sign(&serv_state.current_root);
            let cur_epoch = serv_state.current_epoch;
            match serv_state.multi_sigs.get_mut(&cur_epoch) {
                Some(multi_sig) => {
                    multi_sig.signatures.insert(self.id.clone(), sig);
                    multi_sig.aggregate = aggregate([multi_sig.aggregate, sig].as_slice())?;
                }
                None => {
                    let mut multi_sig = MultiSig {
                        epoch: serv_state.current_epoch,
                        root: serv_state.current_root.clone(),
                        aggregate: sig,
                        signatures: HashMap::new(),
                    };
                    multi_sig.signatures.insert(self.id.clone(), sig);
                    serv_state.multi_sigs.insert(cur_epoch, multi_sig);
                }
            }
            (cur_epoch, sig, serv_state.current_root.clone())
        };
        if let Some(servers) = &self.parallel {
            let mut futures = vec![];
            for (_, srv) in &servers.servers {
                let srv = srv.clone();
                let mut client = srv.get_client().await?;
                let h = head.clone();
                let fut = async move {
                    client
                        .peer_commit(PeerCommitRequest {
                            peer_identity: Some(self.server_identity()),
                            epoch: Some(Epoch { epoch }),
                            signature: sig.as_bytes().to_vec(),
                            head: h,
                        })
                        .await
                };
                futures.push(fut);
            }
            futures::future::join_all(futures).await;
        }
        Ok(())
    }

    pub async fn receive_signatures(
        &self,
        epoch: u64,
        head: &Vec<u8>,
        sig_bytes: &Vec<u8>,
    ) -> Result<()> {
        /// receives the signatures from the parallel servers regarding an epoch. If the contents match
        /// the current root, then the state of multisignature is updated.
        let mut serv_state = self.state.lock().unwrap();
        if epoch != serv_state.current_epoch {
            return Err(anyhow!(
                "Received signatures for an epoch that is not equal to current epoch"
            ));
        }
        if *head != serv_state.current_root {
            return Err(anyhow!(
                "Received signatures for a root that is not equal to current root"
            ));
        }
        let sig = Signature::from_bytes(sig_bytes)?;
        match serv_state.multi_sigs.get_mut(&epoch) {
            Some(multi_sig) => {
                multi_sig.signatures.insert(self.id.clone(), sig);
                multi_sig.aggregate = aggregate([multi_sig.aggregate, sig].as_slice())?;
            }
            None => {
                let mut multi_sig = MultiSig {
                    epoch: serv_state.current_epoch,
                    root: serv_state.current_root.clone(),
                    aggregate: sig,
                    signatures: HashMap::new(),
                };
                multi_sig.signatures.insert(self.id.clone(), sig);
                serv_state.multi_sigs.insert(epoch, multi_sig);
            }
        }
        Ok(())
    }

    pub async fn watch_trigger_prepare(&self) -> Result<()> {
        /// watches the current epoch, and triggers the prepare phase when the epoch interval is reached.
        /// also might trigger prepare if a certain number of transactions are received.
        /// loop every 10 seconds, trigger commit if the epoch interval is reached.
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            self.broadcast_prepare().await?;
        }
    }

    pub async fn trigger_commit(&self) -> Result<()> {
        // triggers the sign and broadcast phase when the prepare phase is finished,
        // and when enough transactions are received.
        // Note: might need to base this on the server configuration
        // TODO: support bulk transactions
        let serv_state = self.state.lock().unwrap();
        let transactions = serv_state
            .transaction_pool
            .get_epoch(serv_state.current_epoch);
        let mut inner_client = self.get_inner_client().await?;
        if let Some(transactions) = transactions {
            for t in transactions {
                inner_client
                    .transaction(TransactionRequest::from(t))
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn receive_peer_transaction(
        &self,
        req: PeerTransactionRequest,
    ) -> Result<Option<()>> {
        // receives a transaction from a peer server, and inserts it into the transaction pool.
        // Note: currently, if the transaction already exists, it is not inserted, and no error message is returned.
        let mut serv_state = self.state.lock().unwrap();
        serv_state.transaction_pool.insert_peer(req)
    }

    pub async fn receive_client_transaction(
        &self,
        req: ClientTransactionRequest,
    ) -> Result<Option<()>> {
        let mut serv_state = self.state.lock().unwrap();
        let epoch = match serv_state.run_state {
            RunState::Prepare(_) => serv_state.current_epoch + 1,
            RunState::Normal => serv_state.current_epoch,
        };
        serv_state.transaction_pool.insert_client(epoch, req)
    }
}
