use super::{MerkleVerseServer, Index};
use crate::grpc_handler::outer;
use anyhow::Result;
use tonic::Status;
use tonic::transport::Channel;
use crate::grpc_handler::inner::MerkleProviderClient;
use crate::config;

impl Default for MerkleVerseServer{
    fn default() -> Self {
        Self {
            inner_dst: "http://[::1]:49563".into(),
            connection_string: "http://[::1]:1319".into(),
            prefix: Index::default(),
            length: 0,
            superior: None,
            parallel: None,
            epoch_interval: 0,
        }
    }
}

impl MerkleVerseServer {
    pub fn relative_index(&self) -> Result<Index> {
        match &self.superior {
            Some(cluster) => {
                let ln =self.prefix.length - cluster.prefix.length;
                Ok(Index{
                    index: self.prefix.index
                        .clone()
                        .as_slice()[usize::try_from(ln)?..]
                        .to_vec(),
                    length: ln,
                })
            }
            None => {
                Ok(Index{
                    index: self.prefix.index.clone(),
                    length: self.prefix.length,
                })
            }
        }
    }

    pub async fn get_client(&self) -> Result<outer::MerkleVerseClient<Channel>> {
        Ok(outer::MerkleVerseClient::connect(self.connection_string.clone()).await?)
    }

    pub async fn get_inner_client(&self) -> Result<MerkleProviderClient<Channel>, Status> {
        match MerkleProviderClient::connect(self.inner_dst.clone()).await {
            Ok(res) => Ok(res),
            Err(e) => Err(Status::internal(format!("Failed to connect to inner server: {}", e))),
        }
    }

    pub async fn from_config(config: config::Server) -> Result<Self> {
       Ok(Self{
           epoch_interval: config.epoch_interval,
           inner_dst: format!("http://[::1]:{}", config.inner_port),
           length: config.length,
           prefix: match config.prefix {
               Some(prefix) => Index::from_b64(&prefix, config.length)?,
               None => Index::default()
           },
           connection_string: format!("http://[::1]:{}", config.outer_port),
           parallel: None,
           superior: None,
       })
    }
}