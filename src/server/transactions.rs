use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use crate::grpc_handler::inner::mversegrpc::transaction_request;
use crate::grpc_handler::outer::mverseouter::{PeerTransactionRequest, ClientTransactionRequest};
use crate::grpc_handler::outer::TransactionRequest;
use crate::server::{Index, PeerServer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionSource {
    Peer(PeerServer),
    Client
}

impl Hash for TransactionSource{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TransactionSource::Peer(_) => "Peer".hash(state),
            TransactionSource::Client => "Client".hash(state),
        }
    }
}

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub enum TransactionOp {
    Register(Index, Vec<u8>),
    Update(Index, Vec<u8>),
    Delete(Index)
}

impl TryFrom<TransactionRequest> for TransactionOp {
    type Error = anyhow::Error;

    fn try_from(request: TransactionRequest) -> Result<Self, Self::Error> {
        Ok(match request.transaction_type() {
                transaction_request::TransactionType::Update => {
                    Self::Update(
                        request.key.into(),
                        request.value.ok_or(anyhow!("Value must be provided for an Update operation!"))?
                    )
                }
                transaction_request::TransactionType::Delete => {
                    Self::Delete(request.key.into())
                }
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Transaction{
    source: TransactionSource,
    operation: TransactionOp,
    auxiliary: Option<Vec<u8>>
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.operation.hash(state);
    }
}

impl Transaction {
    fn from_peer(trans: PeerTransactionRequest, source: PeerServer) -> Result<Self>{
        Ok(Self {
            auxiliary: trans.auxiliary,
            source: TransactionSource::Peer(source),
            operation: TransactionOp::try_from(
                trans.transaction.ok_or(anyhow!("A valid transaction must be provided"))?
            )?
        })
    }

    fn from_client(trans: ClientTransactionRequest) -> Result<Self>{
        Ok(Self{
            auxiliary: trans.auxiliary,
            source: TransactionSource::Client,
            operation: TransactionOp::try_from(
                trans.transaction.ok_or(anyhow!("A valid transaction must be provided"))?
            )?
        })
    }
}

pub struct TransactionPool {
    existence_set: HashMap<u64, HashSet<Transaction>>, // Epoch -> Transactions
}

impl TransactionPool {
    pub fn insert_peer(&mut self, epoch: u64, req: PeerTransactionRequest, source: PeerServer) -> Result<Option<()>>{
        self.insert_transaction(epoch,Transaction::from_peer(req, source)?)
    }

    pub fn insert_client(&mut self, epoch: u64, req: ClientTransactionRequest) -> Result<Option<()>> {
        self.insert_transaction(epoch, Transaction::from_client(req)?)
    }

    fn insert_transaction(&mut self, epoch: u64, transaction: Transaction) -> Result<Option<()>> {
        let haset = match self.existence_set.get_mut(&epoch){
            Some(v) => v,
            None => {
                self.existence_set.insert(epoch, HashSet::new());
                self.existence_set.get_mut(&epoch).unwrap() // this should not fail
            }
        };
        if haset.contains(&transaction){
            return Ok(Some(()));
        }
        haset.insert(transaction);
        Ok(None)
    }
}