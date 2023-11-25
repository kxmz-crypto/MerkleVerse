use crate::server::MerkleVerseServer;
use crate::utils::{b64_to_loc, binary_string};
use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::de::Unexpected::Str;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Server {
    pub id: String,
    pub outer_port: u16,
    pub outer_addr: String,
    pub inner_port: u16,
    pub prefix: Option<String>,
    pub prefix_length: Option<u32>,
    pub length: u32,
    pub epoch_interval: u32, // epoch interval in miliseconds
    pub pub_key: String,
    pub private_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServersConfig {
    pub servers: Vec<Server>,
    pub id: String,
    // TODO: priv_key: String,
}

impl ServersConfig {
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from(path.as_ref()))
            .add_source(Environment::with_prefix("MERKLEVERSE"))
            .build()?;

        s.try_deserialize()
    }
}

impl Server {
    pub fn prefix_bin(&self) -> Result<String> {
        match (&self.prefix, self.prefix_length) {
            (Some(pref), Some(len)) => {
                let len_siz = usize::try_from(len).unwrap();
                Ok(binary_string(&b64_to_loc(&pref, len_siz)?, len_siz))
            }
            _ => Ok(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
