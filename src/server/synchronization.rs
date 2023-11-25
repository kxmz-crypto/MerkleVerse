use crate::server::MerkleVerseServer;
use anyhow::Result;


impl MerkleVerseServer {
   pub async fn get_prefix(&self) -> Result<()> {
       todo!()
   }

    pub async fn get_length(&self) -> Result<()> {
        todo!()
    }

    pub async fn broadcast_prepare(&self) -> Result<()> {
        todo!()
    }

    pub async fn receive_prepare(&self) -> Result<()> {
        todo!()
    }

    pub async fn sign_and_broadcast(&self) -> Result<()> {
        /// Signs the current tree root with BLS, and broadcasts it to the parallel servers.
        todo!()
    }

    pub async fn receive_signatures(&self) -> Result<()> {
        /// receives the signatures from the parallel servers regarding an epoch. If the contents match
        /// the current root, then the state of multisignature is updated.
        todo!()
    }
}