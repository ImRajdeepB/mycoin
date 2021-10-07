use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current unix timestamp.
pub fn now() -> u128 {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    duration.as_secs() as u128 * 1000 + duration.subsec_millis() as u128
}

pub trait Hashable {
    fn bytes(&self) -> Vec<u8>;
    /// Returns the hash of a Hashable.
    ///
    /// For a block, it is the 0x-prefixed lowercase-base16-encoded
    /// SHA256 hash of the JSON-encoded tuple of the predecessor hash, the list of
    /// transactions, the difficulty and the nonce.
    fn hash(&self) -> String {
        format!(
            "0x{}",
            crypto_hash::hex_digest(crypto_hash::Algorithm::SHA256, &self.bytes())
        )
    }
}

mod block;
pub use crate::block::Block;
mod blockchain;
pub use crate::blockchain::{Blockchain, InitGenesis, SubmittedBlock};
mod network;
pub use crate::network::{ChainState, Head, Network};
mod transaction;
pub use crate::transaction::{Output, Transaction};
