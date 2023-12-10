use crate::grpc_handler::inner::mversegrpc::Epoch;
use crate::grpc_handler::outer::mverseouter::{
    ClientTransactionRequest, PeerCommitRequest, PeerPrepareRequest, PeerTransactionRequest,
    ServerIdentity,
};
use crate::grpc_handler::outer::TransactionRequest;
use crate::server::transactions::{Transaction, TransactionPool};
use crate::server::{MerkleVerseServer, ServerId};
use anyhow::{anyhow, Result};
use bls_signatures::{aggregate, Serialize, Signature};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use tokio::time::Instant;
use tokio::sync::watch::{channel, Receiver, Sender};
use tokio::sync::watch::error::RecvError;
use tonic::IntoRequest;
use tracing::instrument;
use tracing::log::Level::Debug;
use crate::grpc_handler::inner::mversegrpc;

const PREPARE_AFTER: u128 = 1000; // try to trigger prepare n milliseconds after the commit
const COMMIT_AFTER: u128 = 1000; // try to trigger commit n milliseconds after the prepare
const LOOP_INTERVAL: u64 = 1000; // epoch watch loop interval in milliseconds
const MIN_TRANSACTIONS: usize = 1; // minimum number of transactions to automatically trigger prepare
const MAX_TRANSACTIONS: usize = 20; // automatically trigger prepare if the number of transactions reaches this number

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

#[derive(Debug)]
pub struct MerkleVerseServerState {
    pub current_root: Vec<u8>,
    pub current_epoch: u64,
    multi_sigs: HashMap<u64, MultiSig>,
    run_state: RunState,
    peer_states: HashMap<ServerId, RunState>,
    transaction_pool: TransactionPool,
    last_commit_time: Option<Instant>,
    last_prepare_time: Option<Instant>,
    prepare_notify: (Sender<u64>, Receiver<u64>),
    commit_notify: (Sender<u64>, Receiver<u64>)
}

impl MerkleVerseServerState {
    pub fn new() -> Self {
        let (prepare_tx, prepare_rx) = channel(0);
        let (commit_tx, commit_rx) = channel(0);
        Self {
            prepare_notify: (prepare_tx, prepare_rx),
            commit_notify: (commit_tx, commit_rx),
            current_root: Default::default(),
            current_epoch: 0,
            multi_sigs: Default::default(),
            run_state: RunState::Normal,
            peer_states: Default::default(),
            transaction_pool: Default::default(),
            last_commit_time: None,
            last_prepare_time: None,
        }
    }
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
            serv_state.last_prepare_time = Some(Instant::now());
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
                                epoch: Some(Epoch { epoch: cur_epoch }),
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
            serv_state.peer_states.insert(server_id, RunState::Prepare(epoch));
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
            tokio::time::sleep(tokio::time::Duration::from_millis(LOOP_INTERVAL)).await;
            let trigger_prep = {
                let serv_state = self
                    .state
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock state"))?;
                if matches!(serv_state.run_state, RunState::Prepare(_)) {
                    continue;
                }
                let t_trigger = match serv_state.last_commit_time {
                    Some(t) => t.elapsed().as_millis() > PREPARE_AFTER,
                    None => true,
                };
                let transaction_cnt = match serv_state
                    .transaction_pool
                    .get_epoch(serv_state.current_epoch)
                {
                    Some(transactions) => transactions.len(),
                    None => 0,
                };
                (t_trigger || transaction_cnt >= MAX_TRANSACTIONS)
                    && transaction_cnt >= MIN_TRANSACTIONS
            };

            if trigger_prep {
                tracing::info!("Triggering prepare phase");
                self.broadcast_prepare().await?;
            }
        }
    }

    pub async fn watch_commit_loop(&self) -> Result<()> {
        /// watches the current epoch, and triggers the commit phase when the epoch interval is reached.
        /// loop every 10 seconds, trigger commit if the epoch interval is reached.
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(LOOP_INTERVAL)).await;
            let trigger_commit = {
                let serv_state = self
                    .state
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock state"))?;
                if matches!(serv_state.run_state, RunState::Normal) {
                    continue;
                }
                let t_trigger = match serv_state.last_prepare_time {
                    Some(t) => t.elapsed().as_millis() > COMMIT_AFTER,
                    None => true,
                };
                t_trigger
            };

            if trigger_commit {
                tracing::info!("Triggering commit phase");
                self.trigger_commit().await?;
            }
        }
    }

    pub async fn routine(&self) -> Result<()> {

        let prep_loop = async move {
            self.watch_trigger_prepare().await
        };

        let commit_loop = async move {
            self.watch_commit_loop().await
        };

        tokio::try_join!(prep_loop, commit_loop)?;
        Ok(())
    }

    pub async fn trigger_commit(&self) -> Result<()> {
        // triggers the sign and broadcast phase when the prepare phase is finished,
        // and when enough transactions are received.
        // Note: might need to base this on the server configuration
        // TODO: support bulk transactions
        tracing::info!("Triggering commit");
        {
            let transactions = {
                let serv_state = self.state.lock().unwrap();
                let res = serv_state
                    .transaction_pool
                    .get_epoch(serv_state.current_epoch);
                match res {
                    None => {vec![]}
                    Some(transet) => {
                        transet
                            .iter().map(|t| TransactionRequest::from(t.clone()))
                            .collect()
                    }
                }
            };
            if transactions.len()>0{
                let mut inner_client = self.get_inner_client().await?;
                for t in transactions {
                    inner_client
                        .transaction(t)
                        .await?;
                }
                inner_client.trigger_epoch(mversegrpc::Empty{}).await?;
            }
        }
        let mut serv_state = self.state.lock().unwrap();
        serv_state.run_state = RunState::Normal;
        serv_state.current_epoch += 1;
        serv_state.commit_notify.0.send(serv_state.current_epoch)?;
        Ok(())
    }

    pub async fn receive_peer_transaction(
        &self,
        req: PeerTransactionRequest,
    ) -> Result<Option<()>> {
        // receives a transaction from a peer server, and inserts it into the transaction pool.
        // Note: currently, if the transaction already exists, it is not inserted, and no error message is returned.
        let mut serv_state = self.state.lock().unwrap();
        self.verify_peer_transaction(&req)?;
        serv_state.transaction_pool.insert_peer(req)
    }

    pub async fn receive_client_transaction(
        &self,
        req: ClientTransactionRequest,
        wait: bool,
    ) -> Result<Option<()>> {
        let r = {
            let mut serv_state = self.state.lock().unwrap();
            let epoch = match serv_state.run_state {
                RunState::Prepare(_) => serv_state.current_epoch + 1,
                RunState::Normal => serv_state.current_epoch,
            };
            let res = serv_state.transaction_pool.insert_client(epoch, &req)?;
            if res.is_some() {
                return Ok(res);
            }
            if let Some(parallels) = &self.parallel {
                let trans = req.transaction.unwrap();
                let signature = self.sign_transaction(&trans)?;
                for (_, ps) in parallels.servers.iter() {
                    let pc = ps.clone();
                    let ts = trans.clone();
                    let sig = signature.clone();
                    let my_id = self.id.0.clone();
                    tokio::spawn(async move {
                        let mut client = pc.get_client().await.unwrap();
                        let res = client
                            .peer_transaction(PeerTransactionRequest {
                                transaction: Some(ts),
                                server_id: my_id,
                                epoch: Some(Epoch { epoch }),
                                signature: sig,
                                auxiliary: None,
                            })
                            .await;
                        if let Err(e) = res {
                            tracing::error!("Failed to send peer transaction to {}: {}", pc.id.0, e);
                        }
                    });
                }
            }
            Ok(res)
        };
        if wait {
            let (mut recv_chan, target_epoch) = {
                let srv_state = self.state.lock().unwrap();
                let target_epoch = match srv_state.run_state {
                    RunState::Prepare(_) => srv_state.current_epoch + 1,
                    RunState::Normal => srv_state.current_epoch,
                };
                let recv_chan = srv_state.commit_notify.1.clone();
                (recv_chan, target_epoch)
            };
            let mut try_cnt = 0;
            while try_cnt<3 {
                match recv_chan.changed().await {
                    Ok(_) if *recv_chan.borrow_and_update() == target_epoch => {
                        return r;
                    }
                    Ok(_) => {
                        try_cnt += 1
                    }
                    Err(e) => {
                        tracing::error!("Failed to receive commit notification: {}", e);
                        return Err(anyhow!("Failed to receive commit notification"));
                    }
                };
            }
            tracing::error!("Failed to receive commit notification");
            Err(anyhow!("Failed to receive commit notification"))
        } else {
            r
        }
    }
}


impl MerkleVerseServerState {
    pub fn add_peer(&mut self, server_id: ServerId) -> Result<()> {
        self.peer_states.insert(
            server_id,
            RunState::Normal
        );
        Ok(())
    }
}
