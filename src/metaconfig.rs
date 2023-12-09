use crate::config::{LocalServerConfig, ServerConfig, ServersConfig};
use crate::server::{PrivateKey};
use crate::utils::b64;
use anyhow::{Context, Result};
use bls_signatures::Serialize as _;
use config::{Config, ConfigError, Environment, File};
use rand::{RngCore};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing_log::LogTracer;

const INNER_BEGIN: i32 = 6000;
const OUTER_BEGIN: i32 = 8000;

#[derive(Debug, Deserialize, Serialize)]
pub struct PeerGroupConfig {
    prefix: Option<String>,
    prefix_length: Option<u32>,
    length: u32,
    count: u32,
    epoch_interval: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetaConfig {
    peer_groups: Vec<PeerGroupConfig>,
}

impl MetaConfig {
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from(path.as_ref()))
            .add_source(Environment::with_prefix("MERKLEVERSE"))
            .build()?;

        s.try_deserialize()
    }

    pub fn to_serv_configs(&self) -> Result<Vec<ServersConfig>> {
        let mut serv_configs = Vec::new();
        let mut s_cnt = 0;
        for peer_group in &self.peer_groups {
            for _ in 0..peer_group.count {
                let port = OUTER_BEGIN + s_cnt;
                let conn_st = format!("127.0.0.1:{}", port);
                let mut rand_buf = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut rand_buf);
                let priv_key = PrivateKey::from(rand_buf);
                let pub_key = priv_key.public_key();
                let serv_config = ServerConfig {
                    prefix: peer_group.prefix.clone(),
                    prefix_length: peer_group.prefix_length,
                    length: peer_group.length,
                    id: format!("srv_{}", s_cnt),
                    connection_string: conn_st.clone(),
                    bls_pub_key: b64(&pub_key.bls.as_bytes()),
                    dalek_pub_key: b64(pub_key.dalek.as_bytes()),
                };
                let local_srv_config = LocalServerConfig {
                    server_config: serv_config,
                    epoch_interval: peer_group.epoch_interval,
                    private_key: b64(priv_key.as_bytes()),
                    inner_port: (INNER_BEGIN + s_cnt) as u16,
                    outer_port: (OUTER_BEGIN + s_cnt) as u16,
                    outer_addr: conn_st,
                };
                serv_configs.push(local_srv_config);
                s_cnt += 1;
            }
        }
        let mut final_conf = Vec::new();
        for i in 0..serv_configs.len() {
            let mut peers = Vec::new();
            for j in 0..serv_configs.len() {
                if j == i {
                    continue;
                }
                peers.push(serv_configs[j].server_config.clone());
            }
            let cur_conf = ServersConfig {
                server: serv_configs[i].clone(),
                peers: Some(peers),
            };
            final_conf.push(cur_conf);
        }
        Ok(final_conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::MerkleVerseServer;
    use anyhow::{anyhow, Result};
    use std::env;

    #[tokio::test]
    async fn test_config() -> Result<()> {
        LogTracer::init().context("Failed to initialize LogTracer")?;
        let path = Path::new(std::module_path!())
            .parent()
            .ok_or(anyhow!("Failed to get parent path"))?
            .join("config")
            .join("cluster.toml");

        let config = MetaConfig::with_path(path)?;
        println!("Number of Clusters: {:#?}", config.peer_groups.len());
        let target_srv = config.to_serv_configs()?;
        println!("Target Servers length: {:#?}", target_srv.len());
        Ok(())
    }
}