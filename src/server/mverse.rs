use super::{MerkleVerseServer, Index};
use crate::grpc_handler::outer;
use anyhow::Result;
use tonic::transport::Channel;

impl Default for MerkleVerseServer{
    fn default() -> Self {
        Self {
            connection_string: "http://[::1]:1319".into(),
            prefix: Index::default(),
            length: 0,
            superior: None,
            parallel: None,
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
}