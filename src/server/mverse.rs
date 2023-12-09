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

pub type PeerServerPointer = Rc<RefCell<PeerServer>>;

impl PeerServer {
    pub async fn get_client(&self) -> Result<outer::MerkleVerseClient<Channel>> {
        Ok(outer::MerkleVerseClient::connect(self.connection_string.clone()).await?)
    }

    pub async fn from_config(config: &config::ServerConfig) -> Result<Self> {
        Ok(Self {
            id: ServerId(config.id.clone()),
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
            connection_string: config.connection_string.clone(),
            public_key: PublicKey::new(
                &general_purpose::STANDARD.decode(&config.bls_pub_key)?,
                &general_purpose::STANDARD.decode(&config.dalek_pub_key)?,
            )?,
        })
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
        tracing::info!(
            "Epoch #{:?} triggered, new head: {:?}",
            res.new_epoch, res.head
        );
        if let Some(servers) = &self.superior {
            let mut futures = vec![];
            for (_, srv) in &servers.servers {
                tracing::info!(
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

    /// Generate from a single Server config
    pub async fn from_config(config: &config::LocalServerConfig) -> Result<Self> {
        let inn_cfig = &config.server_config;
        Ok(Self {
            epoch_interval: config.epoch_interval,
            id: ServerId(inn_cfig.id.clone()),
            inner_dst: format!("http://127.0.0.1:{}", config.inner_port),
            length: inn_cfig.length,
            prefix: match &inn_cfig.prefix {
                Some(prefix) => Index::from_b64(
                    prefix,
                    inn_cfig
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
            public_key: PublicKey::new(
                &general_purpose::STANDARD.decode(&inn_cfig.bls_pub_key)?,
                &general_purpose::STANDARD.decode(&inn_cfig.dalek_pub_key)?,
            )?,
            state: Arc::new(Mutex::new(MerkleVerseServerState::default())),
        })
    }

    /// Generate from a cluster config
    pub async fn from_cluster_config(config: config::ServersConfig) -> Result<Self> {
        let mut prefix_map: HashMap<String, Vec<PeerServerPointer>> = HashMap::new();
        let mut length_map: HashMap<u32, Vec<PeerServerPointer>> = HashMap::new();
        let mut peer_servers: Vec<(String, PeerServerPointer)> = vec![];

        for cfig in config.peers.unwrap_or_default() {
            let pnt: PeerServerPointer =
                Rc::new(RefCell::new(PeerServer::from_config(&cfig).await?));
            peer_servers.push((cfig.id.clone(), pnt.clone()));

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

        peer_servers.sort_by(|a, b| {
            a.1.borrow()
                .prefix
                .to_binstring()
                .unwrap()
                .cmp(&b.1.borrow().prefix.to_binstring().unwrap())
        });

        let mut cur_srv = Self::from_config(&config.server).await?;
        let cur_pref = cur_srv.prefix.to_binstring()?;
        let mut superiors = vec![];
        let mut parallels = vec![];

        for i in 0..peer_servers.len() {
            let peer_server = peer_servers[i].1.borrow();
            let pref = peer_server.prefix.to_binstring()?;
            if peer_server.length == cur_srv.prefix.length && pref.starts_with(&cur_pref) {
                superiors.push(peer_servers[i].1.clone());
            }

            if peer_server.length == cur_srv.length && pref == cur_pref {
                parallels.push(peer_servers[i].1.clone());
            }

            cur_srv
                .state
                .lock()
                .unwrap()
                .add_peer(
                    peer_servers[i].1.clone().borrow().id.clone(),
                )?;
        }

        if !superiors.is_empty() {
            cur_srv.superior = Some(ServerCluster::from(superiors));
        }

        if !parallels.is_empty() {
            cur_srv.parallel = Some(ServerCluster::from(parallels));
        }

        Ok(cur_srv)
    }
}
