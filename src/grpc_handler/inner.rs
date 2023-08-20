
pub use mversegrpc::{merkle_provider_client::MerkleProviderClient};

pub mod mversegrpc {
    tonic::include_proto!("mversegrpc");
}