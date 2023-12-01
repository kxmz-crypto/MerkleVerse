use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use crate::grpc_handler::inner::mversegrpc::transaction_request;
use crate::grpc_handler::outer::mverseouter::{PeerTransactionRequest, ClientTransactionRequest};
use crate::grpc_handler::outer::TransactionRequest;
use crate::server::{Index, PeerServer, ServerId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionSource {
    Peer(ServerId),
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

impl TransactionOp{
    pub fn to_raw(&self) -> (Vec<u8>, Option<Vec<u8>>) {
        match self {
            Self::Register(index, value) => (index.index.clone(), Some(value.clone())),
            Self::Update(index, value) => (index.index.clone(), Some(value.clone())),
            Self::Delete(index) => (index.index.clone(), None),
        }
    }
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
    pub source: TransactionSource,
    pub operation: TransactionOp,
    pub auxiliary: Option<Vec<u8>>
}

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.operation.hash(state);
    }
}

impl Transaction {
    fn from_peer(trans: PeerTransactionRequest, source: ServerId) -> Result<Self>{
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

impl From<&Transaction> for TransactionRequest {
    fn from(transaction: &Transaction) -> Self {
        let (key, value) = transaction.operation.to_raw();
        Self {
            key,
            value,
            origin: match &transaction.source {
                TransactionSource::Peer(id) => transaction_request::Origin::Server.into(),
                TransactionSource::Client => transaction_request::Origin::Client.into(),
            },
            transaction_type: match &transaction.operation {
                TransactionOp::Register(_, _) => transaction_request::TransactionType::Update.into(),
                TransactionOp::Update(_, _) => transaction_request::TransactionType::Update.into(),
                TransactionOp::Delete(_) => transaction_request::TransactionType::Delete.into(),
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct TransactionPool {
    existence_set: HashMap<u64, HashSet<Transaction>>, // Epoch -> Transactions
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            existence_set: HashMap::new()
        }
    }

    pub fn insert_peer(&mut self, req: PeerTransactionRequest, source: ServerId) -> Result<Option<()>>{
        self.insert_transaction(
            req.epoch.clone().ok_or(anyhow!("An epoch number must be provided!"))?.epoch,
            Transaction::from_peer(req, source)?
        )
    }

    pub fn insert_client(&mut self, epoch: u64, req: ClientTransactionRequest) -> Result<Option<()>> {
        self.insert_transaction(epoch, Transaction::from_client(req)?)
    }

    pub fn get_epoch(&self, epoch: u64) -> Option<&HashSet<Transaction>> {
        self.existence_set.get(&epoch)
    }

    pub fn purge_before(&mut self, epoch: u64) -> Result<()> {
        let mut to_remove = vec![];
        for (ep, _) in self.existence_set.iter(){
            if *ep < epoch {
                to_remove.push(*ep);
            }
        }
        for ep in to_remove{
            self.existence_set.remove(&ep);
        }
        Ok(())
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
            return Ok(Some(())); // the transaction exists
        }
        haset.insert(transaction);
        Ok(None)
    }
}