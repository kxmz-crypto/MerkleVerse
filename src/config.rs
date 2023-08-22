use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
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
    //TODO: pub_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ServersConfig {
    servers: Vec<Server>,
    id: String,
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

#[cfg(test)]
mod tests{
    use super::*;
    use anyhow::{Result, anyhow};
    use std::env;

    #[test]
    fn test_config() -> Result<()>{
        let path = Path::new( std::module_path!())
            .parent().ok_or(anyhow!("Failed to get parent path"))?
            .join("config")
            .join("poc.toml");

        env::set_var("MERKLEVERSE_ID", "edge1");
        let config = ServersConfig::with_path(path)?;
        println!("{:?}", config);
        Ok(())
    }
}