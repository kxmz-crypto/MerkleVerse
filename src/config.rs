use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Server {
    id: String,
    outer_port: u16,
    outer_addr: String,
    inner_port: u16,
    prefix: Option<String>,
    prefix_length: Option<u32>,
    length: u32,
    epoch_interval: u32, // epoch interval in miliseconds
    //TODO: pub_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ServersConfig {
    servers: Vec<Server>,
    id: String,
    // TODO: priv_key: String,
}

impl ServersConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/servers"))
            .add_source(Environment::with_prefix("MERKLEVERSE"))
            .build()?;

        s.try_deserialize()
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from(path.as_ref()))
            .add_source(Environment::with_prefix("MERKLEVERSE"))
            .build()?;

        s.try_deserialize()
    }
}