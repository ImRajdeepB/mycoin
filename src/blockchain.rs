use super::{Block, Output};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A helper struct used to parse json inputs to submit a new block to the chain.
#[derive(Serialize, Deserialize)]
pub struct SubmittedBlock {
    pub block: Block,
}

/// A helper struct used to parse json inputs to initialize the chain.
#[derive(Serialize, Deserialize)]
pub struct InitGenesis {
    pub init: Block,
}

/// A struct that keeps track of a single chain in the [network](struct.Network.html).
///
/// The `Blockchain` is analogous to a single chain (or fork) in a network.
/// In this client, the methods of `Blockchain` are invoked from the methods in [Network](struct.Network.html#impl).
#[derive(Clone)]
pub struct Blockchain {
    /// A list of all the blocks and their creation timestamps in the chain.
    pub blocks: Vec<(Block, u128)>,
    /// It stores the block hashes of all the blocks in the chain.
    pub blocks_set: HashSet<String>,
    /// A list of the unspent transaction outputs in the chain.
    pub outputs: Vec<Output>,
    /// It stores the unspent transaction outputs in the chain.
    pub outputs_set: HashSet<Output>,
}

impl Blockchain {
    /// Creates a new `Blockchain` instance.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let blockchain = Blockchain::new();
    /// ```
    pub fn new() -> Self {
        Blockchain {
            blocks: vec![],
            blocks_set: HashSet::new(),
            outputs: vec![],
            outputs_set: HashSet::new(),
        }
    }
    /// Initializes the chain with the genesis block.
    ///
    /// Returns `true` if the chain was initialized successful, otherwise returns `false`.
    pub fn init(&mut self, block: Block, timestamp: u128) -> bool {
        // init genesis block and return true/false

        let mut blocks_spent: HashSet<Output> = HashSet::new();
        let mut blocks_created: HashSet<Output> = HashSet::new();
        for transaction in &block.transactions {
            blocks_spent.extend(transaction.inputs());
            blocks_created.extend(transaction.outputs());
        }

        self.outputs_set
            .retain(|output| !blocks_spent.contains(output));
        self.outputs_set.extend(blocks_created);
        self.outputs = self.outputs_set.clone().into_iter().collect();

        self.blocks.push((block.clone(), timestamp));
        self.blocks_set.insert(block.hash.to_owned());
        true
    }
    /// Submits a new block to the chain.
    ///
    /// Returns `true` if the block was added successful, otherwise returns `false`.
    pub fn submit(&mut self, block: Block, timestamp: u128) -> bool {
        let mut blocks_spent: HashSet<Output> = HashSet::new();
        let mut blocks_created: HashSet<Output> = HashSet::new();
        for transaction in &block.transactions {
            let inputs = transaction.inputs();
            if !(&inputs - &self.outputs_set).is_empty() || !(&inputs & &blocks_spent).is_empty() {
                println!("{{\"error\":\"invalid transaction\"}}");
                return false;
            }

            let input_value = transaction.input_value();
            let output_value = transaction.output_value();
            if output_value != input_value {
                println!("{{\"error\":\"invalid transaction\"}}");
                return false;
            }
            blocks_spent.extend(inputs);
            blocks_created.extend(transaction.outputs());
        }

        self.outputs_set
            .retain(|output| !blocks_spent.contains(output));
        self.outputs_set.extend(blocks_created);
        self.outputs = self.outputs_set.clone().into_iter().collect();

        self.blocks.push((block.clone(), timestamp));
        self.blocks_set.insert(block.hash.to_owned());
        true
    }
}
