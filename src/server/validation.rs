use crate::grpc_handler::inner::mversegrpc;
use crate::grpc_handler::outer::mverseouter::PeerTransactionRequest;

use crate::server::MerkleVerseServer;
use anyhow::{anyhow, Result};
use bls_signatures::Serialize;
use ed25519_dalek::{Signer, Verifier};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

// For public keys, the bls and dalek keys are specified separately.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PublicKey {
    pub bls: bls_signatures::PublicKey,
    pub dalek: ed25519_dalek::VerifyingKey,
}

// For private keys, the bls and dalek private keys are derived deterministically from the raw key.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrivateKey {
    raw: Vec<u8>,
    pub bls: bls_signatures::PrivateKey,
    pub dalek: ed25519_dalek::SigningKey,
}

impl PrivateKey {
    pub fn as_bytes(&self) -> &[u8] {
        self.raw.as_slice()
    }
}

impl TryFrom<Vec<u8>> for PrivateKey {
    type Error = anyhow::Error;
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        if value.len()<32 {
            return Err(anyhow!("Private key invalid length"));
        }
        let vals: [u8;32] = value.as_slice().try_into()?;
        Ok(Self::from(vals))
    }
}

impl From<[u8; 32]> for PrivateKey {
    fn from(value: [u8; 32]) -> Self {
        Self {
            bls: bls_signatures::PrivateKey::new(&value),
            dalek: ed25519_dalek::SigningKey::from_bytes(
                &value
            ),
            raw: value.to_vec(),
        }
    }
}

impl PrivateKey {
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            bls: self.bls.public_key(),
            dalek: self.dalek.verifying_key(),
        }
    }
}

impl PublicKey {
    pub fn new(bls: &Vec<u8>, dalek: &Vec<u8>) -> Result<Self> {
        Ok(Self {
            bls: bls_signatures::PublicKey::from_bytes(bls)?,
            dalek: ed25519_dalek::VerifyingKey::try_from(&dalek[0..32])?,
        })
    }
}

impl Hash for mversegrpc::TransactionRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
        self.transaction_type.hash(state);
    }
}

impl MerkleVerseServer {
    pub fn verify_peer_transaction(&self, transaction: &PeerTransactionRequest) -> Result<()> {
        // verifies the peer transaction with dalek
        let parallel = match &self.parallel {
            None => Err(anyhow!("Peer server cluster does not exist!")),
            Some(p) => anyhow::Ok(p),
        }?;

        let peer = parallel
            .get_server(&transaction.server_id.clone().into())
            .ok_or(anyhow!("Peer server {} does not exist in server cluster {:?}!", &transaction.server_id, parallel))?;

        let target_transaction = match &transaction.transaction {
            None => Err(anyhow!("Transaction must be provided!")),
            Some(t) => anyhow::Ok(t),
        }?;

        let peer_key = peer.public_key.dalek;
        let peer_sig = ed25519_dalek::Signature::try_from(&transaction.signature[..])?;
        let mut hasher = DefaultHasher::new();
        target_transaction.hash(&mut hasher);
        let msg = hasher.finish().to_le_bytes();
        peer_key
            .verify(&msg, &peer_sig)
            .map_err(|e| anyhow!("Peer transaction signature is invalid: {}", e))
    }

    pub fn sign_transaction(
        &self,
        transaction: &mversegrpc::TransactionRequest,
    ) -> Result<Vec<u8>> {
        // signs the transaction with dalek
        let mut hasher = DefaultHasher::new();
        transaction.hash(&mut hasher);
        let msg = hasher.finish().to_le_bytes();
        let sig = self.private_key.dalek.sign(&msg);
        Ok(sig.to_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{RngCore, thread_rng};

    #[test]
    fn tst_privkey_gen() -> Result<()>{
        let mut src_bytes = [0u8;32];
        thread_rng().fill_bytes(&mut src_bytes);
        let priv_key = PrivateKey::from(src_bytes);
        Ok(())
    }
}
