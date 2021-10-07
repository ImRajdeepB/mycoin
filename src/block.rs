use super::{Hashable, Transaction};
use serde::{Deserialize, Serialize};

/// A block contains the predecessor block hash, a list of transactions, the target
/// difficulty, the nonce, and the block’s hash.
#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub difficulty: u32,
    pub hash: String,
    pub nonce: u64,
    pub predecessor: String,
    pub transactions: Vec<Transaction>,
}

impl Hashable for Block {
    /// Returns an encoded version of a block, which would later be hashed.
    fn bytes(&self) -> Vec<u8> {
        let mut ms = String::from("[\"");

        ms.push_str(&self.predecessor);
        ms.push_str("\",[");

        let txnslen = self.transactions.len();

        for (i, txn) in self.transactions.iter().enumerate() {
            ms.push_str("{\"inputs\":[");
            let iplen = txn.inputs.len();
            for (ip_i, ip) in txn.inputs.iter().enumerate() {
                ms.push_str(&format!("{{\"amount\":{},\"id\":{}}}", ip.amount, ip.id));
                if ip_i < iplen - 1 {
                    ms.push_str(",");
                }
            }
            ms.push_str("],");
            ms.push_str("\"outputs\":[");
            let oplen = txn.outputs.len();
            for (op_i, op) in txn.outputs.iter().enumerate() {
                ms.push_str(&format!("{{\"amount\":{},\"id\":{}}}", op.amount, op.id));
                if op_i < oplen - 1 {
                    ms.push_str(",");
                }
            }
            ms.push_str("]}");
            if i < txnslen - 1 {
                ms.push_str(",");
            }
        }
        ms.push_str("],");
        ms.push_str(&format!("{},{}", &self.difficulty, &self.nonce));
        ms.push_str("]");

        let mut bss = vec![];
        bss.extend(ms.as_bytes());
        bss
    }
}

impl Block {
    pub fn new(
        difficulty: u32,
        hash: String,
        nonce: u64,
        predecessor: String,
        transactions: Vec<Transaction>,
    ) -> Self {
        Block {
            difficulty,
            hash,
            nonce,
            predecessor,
            transactions,
        }
    }
    /// Validates if the submitted block was mined correctly.
    ///
    /// `validate` checks whether:
    /// * the provided hash is same as the computed hash.
    /// * the provided value of difficulty is within the limits.
    /// * the number of leading zeroes in block hash matches the difficulty.
    ///
    /// Returns `true` if valid, otherwise returns false.
    pub fn validate(&mut self) -> bool {
        let computed_hash = self.hash();
        if self.hash != computed_hash {
            println!("{{\"error\":\"invalid hash\"}}");
            return false;
        }

        if self.difficulty > 64 {
            // difficulty can't be greater than 64 since the hash
            // has 64 characters (excluding the prefix "0x").
            println!("{{\"error\":\"maximum value of difficulty is 64\"}}");
            return false;
        }
        let (hash_start, _) = computed_hash.split_at((2 + self.difficulty) as usize);
        if hash_start == format!("0x{}", "0".repeat(self.difficulty as usize)) {
            return true;
        }
        println!("{{\"error\":\"leading zeroes in block hash did not match difficulty\"}}");
        return false;
    }
}
