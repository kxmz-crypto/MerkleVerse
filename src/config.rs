use crate::utils::{b64_to_loc, binary_string};
use anyhow::Result;
use config::{Config, ConfigBuilder, ConfigError, Environment, File};

use serde::{Deserialize, Serialize, Serializer};

use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ServerConfig {
    pub connection_string: String,
    pub id: String,
    pub prefix: Option<String>,
    pub prefix_length: Option<u32>,
    pub length: u32,
    pub bls_pub_key: String,
    pub dalek_pub_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocalServerConfig {
    #[serde(flatten)]
    pub server_config: ServerConfig,
    pub outer_port: u16,
    pub outer_addr: String,
    pub inner_port: u16,
    pub private_key: String,
    pub epoch_interval: u32, // epoch interval in miliseconds
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServersConfig {
    pub server: LocalServerConfig,
    #[serde(flatten)]
    pub peers: Option<Vec<ServerConfig>>,
}

impl ToString for ServersConfig {
    fn to_string(&self) -> String {
        toml::to_string(&self).unwrap()
    }
}

impl ServersConfig {
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from(path.as_ref()))
            .add_source(Environment::with_prefix("MERKLEVERSE"))
            .build()?;

        s.try_deserialize()
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let res = toml::to_string(&self)?;
        std::fs::write(path, res)?;
        Ok(())
    }
}

fn prefix_bin(prefix: &Option<String>, length: &Option<u32>) -> Result<String> {
    match (prefix, length) {
        (Some(pref), Some(len)) => {
            let len_siz = usize::try_from(*len).unwrap();
            Ok(binary_string(&b64_to_loc(pref, len_siz)?, len_siz))
        }
        _ => Ok(String::new()),
    }
}

impl ServerConfig {
    pub fn prefix_bin(&self) -> Result<String> {
        prefix_bin(&self.prefix, &self.prefix_length)
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
        let path = Path::new(std::module_path!())
            .parent()
            .ok_or(anyhow!("Failed to get parent path"))?
            .join("config")
            .join("poc.toml");

        env::set_var("MERKLEVERSE_ID", "edge1");
        let config = ServersConfig::with_path(path)?;
        println!("{:#?}", config);
        let target_srv = MerkleVerseServer::from_cluster_config(config).await?;
        println!("{:#?}", target_srv);
        Ok(())
    }
}
