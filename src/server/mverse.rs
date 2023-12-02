use super::{Index, MerkleVerseServer, PrivateKey, PublicKey, ServerId};
use crate::config;
use crate::grpc_handler::inner::mversegrpc;
use crate::grpc_handler::inner::MerkleProviderClient;
use crate::grpc_handler::outer;
use crate::grpc_handler::outer::mverseouter::ClientTransactionRequest;
use crate::server::synchronization::MerkleVerseServerState;
use crate::server::{PeerServer, ServerCluster};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tonic::transport::Channel;
use tonic::{IntoRequest, Status};

pub type MServerPointer = Rc<RefCell<MerkleVerseServer>>;

impl PeerServer {
    pub async fn get_client(&self) -> Result<outer::MerkleVerseClient<Channel>> {
        Ok(outer::MerkleVerseClient::connect(self.connection_string.clone()).await?)
    }
}

impl MerkleVerseServer {
    pub fn relative_index(&self, superior: Option<&PeerServer>) -> Result<Index> {
        match superior {
            Some(srv) => {
                let ln = self.prefix.length - srv.prefix.length;
                Ok(Index {
                    index: self.prefix.index.clone().as_slice()[usize::try_from(ln)?..].to_vec(),
                    length: ln,
                })
            }
            None => Ok(Index {
                index: self.prefix.index.clone(),
                length: self.prefix.length,
            }),
        }
    }

    pub async fn get_client(&self) -> Result<outer::MerkleVerseClient<Channel>> {
        Ok(outer::MerkleVerseClient::connect(self.connection_string.clone()).await?)
    }

    pub async fn get_inner_client(&self) -> Result<MerkleProviderClient<Channel>, Status> {
        match MerkleProviderClient::connect(self.inner_dst.clone()).await {
            Ok(res) => Ok(res),
            Err(e) => Err(Status::internal(format!(
                "Failed to connect to inner server: {}",
                e
            ))),
        }
    }

    async fn trigger_epoch(&self) -> Result<()> {
        let mut inn_client = self.get_inner_client().await?;
        let res = inn_client
            .trigger_epoch(mversegrpc::Empty {}.into_request())
            .await?
            .into_inner();
        eprintln!(
            "Epoch #{:?} triggered, new head: {:?}",
            res.new_epoch, res.head
        );
        if let Some(servers) = &self.superior {
            let mut futures = vec![];
            for (_, srv) in &servers.servers {
                eprintln!(
                    "Triggering epoch on superior server: {:?}",
                    srv.connection_string
                );
                let mut client = srv.get_client().await?;
                let cphead = res.head.clone();
                futures.push(async move {
                    client
                        .client_transaction(
                            ClientTransactionRequest {
                                transaction: Some(mversegrpc::TransactionRequest {
                                    value: Some(cphead),
                                    key: self.relative_index(Some(srv)).unwrap().index,
                                    transaction_type:
                                        mversegrpc::transaction_request::TransactionType::Update
                                            .into(),
                                }),
                                auxiliary: None, // TODO: make auxiliary a signature of the head
                            }
                            .into_request(),
                        )
                        .await
                });
            }
            futures::future::try_join_all(futures).await?;
        }
        Ok(())
    }

    pub async fn epoch_loop(&self) -> Result<()> {
        eprintln!("Epoch loop started");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.epoch_interval.into(),
            ))
            .await;
            self.trigger_epoch().await?;
        }
    }

    /// Generate from a single Server config
    pub async fn from_config(config: &config::Server) -> Result<Self> {
        Ok(Self {
            epoch_interval: config.epoch_interval,
            id: ServerId(config.id.clone()),
            inner_dst: format!("http://127.0.0.1:{}", config.inner_port),
            length: config.length,
            prefix: match &config.prefix {
                Some(prefix) => Index::from_b64(
                    prefix,
                    config
                        .prefix_length
                        .ok_or(anyhow!("Prefix length not specified"))?,
                )?,
                None => Index::default(),
            },
            connection_string: format!("127.0.0.1:{}", config.outer_port),
            parallel: None,
            superior: None,
            private_key: PrivateKey::try_from(
                general_purpose::STANDARD.decode(&config.private_key)?,
            )?,
            public_key: PublicKey::try_from(general_purpose::STANDARD.decode(&config.pub_key)?)?,
            state: Arc::new(Mutex::new(MerkleVerseServerState::default())),
        })
    }

    /// Generate from a cluster config
    pub async fn from_cluster_config(config: config::ServersConfig) -> Result<Self> {
        let mut prefix_map: HashMap<String, Vec<MServerPointer>> = HashMap::new();
        let mut length_map: HashMap<u32, Vec<MServerPointer>> = HashMap::new();
        let mut servers: Vec<(String, MServerPointer)> = vec![];

        for cfig in config.servers {
            let pnt: MServerPointer = Rc::new(RefCell::new(Self::from_config(&cfig).await?));
            servers.push((cfig.id.clone(), pnt.clone()));

            let prefix_bin = &cfig.prefix_bin()?;
            if prefix_map.get(prefix_bin).is_none() {
                prefix_map.insert(prefix_bin.clone(), vec![]);
            }
            prefix_map.get_mut(prefix_bin).unwrap().push(pnt.clone());

            if length_map.get(&cfig.length).is_none() {
                length_map.insert(cfig.length, vec![]);
            }
            length_map.get_mut(&cfig.length).unwrap().push(pnt.clone());
        }

        servers.sort_by(|a, b| {
            a.1.borrow()
                .prefix
                .to_binstring()
                .unwrap()
                .cmp(&b.1.borrow().prefix.to_binstring().unwrap())
        });

        let mut res = None;
        for i in 0..servers.len() {
            let mut server = servers[i].1.borrow_mut();
            let pref = server.prefix.to_binstring()?;
            let mut superiors: Vec<MServerPointer> = vec![];
            for j in 0..i {
                let sup = &servers[j].1.borrow();
                if sup.length == server.prefix.length
                    && pref.starts_with(&sup.prefix.to_binstring()?)
                {
                    superiors.push(servers[j].1.clone());
                }
            }
            if !superiors.is_empty() {
                server.superior = Some(ServerCluster::from(superiors));
            }

            if &servers[i].0 == &config.id {
                res = Some(servers[i].1.clone());
            }
        }

        if let Some(s) = res {
            let mut server = s.borrow_mut();
            let mut parallels: Vec<MServerPointer> = vec![];
            let prf = server.prefix.to_binstring()?;
            for ns in prefix_map.get(&prf).unwrap() {
                if !Rc::ptr_eq(ns, &s) && ns.borrow().prefix.length == server.prefix.length {
                    parallels.push(ns.clone());
                }
            }

            if !parallels.is_empty() {
                server.parallel = Some(ServerCluster::from(parallels));
            }

            Ok(server.clone())
        } else {
            Err(anyhow!("Target not found in config"))
        }
    }
}
