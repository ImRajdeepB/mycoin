use super::{now, Block, Blockchain, Output};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};

/// A struct that represents a head (possible fork) in the network.
#[allow(non_snake_case)]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Head {
    pub height: u64,
    pub totalWork: u64,
    pub hash: String,
}

/// A struct that represents the state of the network.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ChainState {
    pub height: u64,
    pub totalWork: u64,
    pub hash: String,
    pub outputs: Vec<Output>,
}

/// A struct that keeps track of the whole network.
///
/// The `Network` stores the blocks in the main chain, possible forks, and the overall state.
/// The user interacts with the methods of this struct.
pub struct Network {
    /// Maximum number of blocks in [recent_blocks_queue](#structfield.recent_blocks_queue).
    pub recent_count_limit: usize,
    /// It maps the block hash of the recent blocks in the main chain with their corresponding height, creation timestamp, totalWork, the block instance, outputs_set, and outputs.
    ///
    /// This acts as a cache which allows users to create forks from recent blocks quickly without
    /// parsing the whole chain (or storing unspent outputs of older blocks of the chain).
    pub recent_blocks: HashMap<String, (u64, u128, u64, Block, HashSet<Output>, Vec<Output>)>,
    /// A queue keeping track of the recent blocks.
    pub recent_blocks_queue: VecDeque<String>,
    /// It stores a copy of each of the possible forks by mapping the latest block hash of the fork with their corresponding height, creation timestamp, totalWork, and the chain instance.
    pub forks: HashMap<String, (u64, u128, u64, Blockchain)>,
    /// It stores the head of each of the possible forks.
    pub heads: HashSet<Head>,
    /// A list of all the blocks and their creation timestamps in the main chain.
    pub blocks: Vec<(Block, u128)>,
    /// It stores the block hashes of all the blocks in the main chain.
    pub blocks_set: HashSet<String>,
    /// The current state of the network.
    pub state: ChainState,
}

impl Network {
    /// Creates a new `Network` instance.
    ///
    /// # Arguments
    ///
    /// * `recent_count_limit` - Maximum number of blocks allowed in the recent blocks in the network.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let network = Network::new(2);
    /// ```
    pub fn new(recent_count_limit: usize) -> Self {
        Network {
            recent_count_limit: recent_count_limit,
            recent_blocks: HashMap::new(),
            recent_blocks_queue: VecDeque::new(),
            forks: HashMap::new(),
            heads: HashSet::new(),
            blocks: vec![],
            blocks_set: HashSet::new(),
            state: ChainState {
                height: 0,
                totalWork: 0,
                hash: String::from(""),
                outputs: vec![],
            },
        }
    }
    /// Returns the blocks and the current state of the main chain.
    ///
    /// The fork choice rule is:
    /// * The longest chain is chosen.
    /// * If any two chains have the same height, the one with highest `totalWork` is chosen.
    /// * If the chains have equal [totalWork](struct.Head.html#structfield.totalWork), the chain head which was created earlier is chosen.
    pub fn get_main_chain(&mut self) -> (Vec<(Block, u128)>, HashSet<String>, ChainState) {
        if self.heads.len() == 0 {
            return (
                vec![],
                HashSet::new(),
                ChainState {
                    height: 0,
                    totalWork: 0,
                    hash: String::from(""),
                    outputs: vec![],
                },
            );
        }

        let mut max_total_work = 0;
        let mut max_total_work_head_hash = String::from("");
        for h in &self.heads {
            if h.totalWork > max_total_work {
                max_total_work = h.totalWork;
                max_total_work_head_hash = h.hash.to_owned();
            }
        }

        let mut heads_with_largest_total_work = vec![];
        for h in &self.heads {
            if h.totalWork == max_total_work {
                heads_with_largest_total_work.push(Head {
                    hash: h.hash.to_owned(),
                    totalWork: h.totalWork,
                    height: h.height,
                });
            }
        }

        if heads_with_largest_total_work.len() > 1 {
            // multiple heads with the same max totalWork
            let mut oldest_timestamp = u128::max_value();
            let mut selected_head_hash = String::from("");
            for h in heads_with_largest_total_work {
                let (_height, created_at, _total_work, _fork) =
                    self.forks.get(&h.hash.to_owned()).unwrap();
                if *created_at < oldest_timestamp {
                    oldest_timestamp = *created_at;
                    selected_head_hash = h.hash;
                }
            }
            max_total_work_head_hash = selected_head_hash;
        }

        let (height, _created_at, total_work, main_chain) = self
            .forks
            .get(&max_total_work_head_hash.to_owned())
            .unwrap();

        let blocks = &main_chain.blocks;
        let blocks_set = main_chain.blocks_set.clone();

        (
            blocks.clone(),
            blocks_set,
            ChainState {
                height: *height,
                totalWork: *total_work,
                hash: max_total_work_head_hash,
                outputs: main_chain.outputs.clone(),
            },
        )
    }
    /// Computes the blockchain at a particular block, returns the chain and totalWork done till that block.
    ///
    /// # Arguments
    ///
    /// * `hash` - hash of the block.
    ///
    /// The main purpose of this method is to compute the unspent outputs at a
    /// particular block (which is usually an older block not present in [recent_blocks](#structfield.recent_blocks)).
    pub fn compute_chain_at_block(&mut self, hash: String) -> (Blockchain, u64) {
        let mut total_work = 0;
        let mut blocks: Vec<(Block, u128)> = vec![];
        let mut blocks_set = HashSet::new();
        let mut utxos = HashSet::new();
        let mut outputs: Vec<Output> = vec![];
        let mut is_referred_block = false;
        for block in &self.blocks {
            if block.0.hash == hash {
                is_referred_block = true;
            }
            let mut blocks_spent: HashSet<Output> = HashSet::new();
            let mut blocks_created: HashSet<Output> = HashSet::new();
            for transaction in &block.0.transactions {
                blocks_spent.extend(transaction.inputs());
                blocks_created.extend(transaction.outputs());
            }

            blocks.push(block.clone());
            blocks_set.insert(block.0.hash.to_owned());
            utxos.retain(|output| !blocks_spent.contains(output));
            utxos.extend(blocks_created);
            outputs = utxos.clone().into_iter().collect();
            total_work += u64::pow(16, block.0.difficulty);
            if is_referred_block {
                break;
            }
        }
        (
            Blockchain {
                blocks: blocks,
                blocks_set: blocks_set,
                outputs: outputs,
                outputs_set: utxos,
            },
            total_work,
        )
    }
    /// Creates a new genesis block.
    ///
    /// Returns `true` if the block was added successful, otherwise returns `false`.
    pub fn init(&mut self, block: Block) -> bool {
        let mut blockchain = Blockchain::new();
        let bhash = block.hash.to_owned();
        if self.forks.contains_key(&bhash) || self.blocks_set.contains(&bhash) {
            println!("{{\"error\":\"duplicate hash\"}}");
            return false;
        }
        let total_work = u64::pow(16, block.difficulty);
        let timestamp = now();
        if !blockchain.init(block.clone(), timestamp) {
            return false;
        }
        self.forks.insert(
            bhash.to_owned(),
            (1, timestamp, total_work, blockchain.clone()),
        );
        self.heads.insert(Head {
            height: 1,
            totalWork: total_work,
            hash: bhash.to_owned(),
        });
        let (main_chain_blocks, main_chain_blocks_set, main_chain_state) = self.get_main_chain();
        self.blocks = main_chain_blocks;
        self.blocks_set = main_chain_blocks_set;
        self.state = main_chain_state;
        // If the block was added to the main chain
        if self.blocks.last().unwrap().0.hash.to_owned() == bhash.to_owned() {
            self.recent_blocks.insert(
                bhash.to_owned(),
                (
                    1,
                    timestamp,
                    total_work,
                    block,
                    blockchain.outputs_set.clone(),
                    blockchain.outputs.clone(),
                ),
            );
            if self.recent_blocks_queue.len() == self.recent_count_limit {
                if let Some(v) = self.recent_blocks_queue.pop_front() {
                    self.recent_blocks.remove(&v.to_owned());
                }
            }
            self.recent_blocks_queue.push_back(bhash.to_owned());
        }
        println!("{{\"ok\":[]}}");

        true
    }
    /// Submits a new block to the network.
    ///
    /// Returns `true` if the block was added successful, otherwise returns `false`.
    pub fn submit(&mut self, block: Block) -> bool {
        if self.heads.len() == 0 {
            println!("{{\"error\":\"must initialize first\"}}");
            return false;
        }

        let hash_copy = block.hash.to_owned();
        let (hash_start, _) = hash_copy.split_at((2 + block.difficulty) as usize);
        if hash_start != format!("0x{}", "0".repeat(block.difficulty as usize)) {
            println!("{{\"error\":\"leading zeroes in block hash did not match difficulty\"}}");
            return false;
        }

        let bhash = block.hash.to_owned();
        let predecessor_hash = block.predecessor.to_owned();
        if !self.forks.contains_key(&predecessor_hash)
            && !self.blocks_set.contains(&predecessor_hash)
        {
            println!("{{\"error\":\"no predecessor found\"}}");
            return false;
        }
        if self.forks.contains_key(&bhash) || self.blocks_set.contains(&bhash) {
            println!("{{\"error\":\"duplicate hash\"}}");
            return false;
        }

        let predecessor_height;
        let mut _predecessor_created_at = 0 as u128;
        let predecessor_total_work;
        let mut chain: Blockchain;

        if self.forks.contains_key(&predecessor_hash) {
            // predecessor is a head
            let (
                tmp_predecessor_height,
                tmp_predecessor_created_at,
                tmp_predecessor_total_work,
                tmp_chain,
            ) = self.forks.get(&predecessor_hash).unwrap();
            predecessor_height = *tmp_predecessor_height;
            _predecessor_created_at = *tmp_predecessor_created_at;
            predecessor_total_work = *tmp_predecessor_total_work;
            chain = tmp_chain.clone();
        } else {
            // predecessor is not a head (but is a block in the main chain)

            if self.recent_blocks.contains_key(&predecessor_hash) {
                // predecessor is within the last `recent_count_limit` blocks
                let (
                    tmp_predecessor_height,
                    tmp_predecessor_created_at,
                    tmp_predecessor_total_work,
                    _tmp_block,
                    tmp_outputs_set,
                    tmp_outputs,
                ) = self.recent_blocks.get(&predecessor_hash).unwrap();
                predecessor_height = *tmp_predecessor_height;
                _predecessor_created_at = *tmp_predecessor_created_at;
                predecessor_total_work = *tmp_predecessor_total_work;

                let sb;
                if self.blocks.len() >= predecessor_height as usize {
                    sb = self.blocks[0..predecessor_height as usize].to_vec();
                } else {
                    sb = self.blocks.clone();
                }
                let mut blocks_to_exclude: HashSet<String> = HashSet::new();
                for rb in &self.recent_blocks_queue {
                    let (h, _, _, _, _, _) = self.recent_blocks.get(&rb.to_owned()).unwrap();
                    if *h <= predecessor_height {
                        blocks_to_exclude.insert(rb.to_owned());
                    }
                }
                chain = Blockchain {
                    blocks: sb,
                    blocks_set: &self.blocks_set.clone() - &blocks_to_exclude.clone(),
                    outputs: tmp_outputs.clone(),
                    outputs_set: tmp_outputs_set.clone(),
                }
            } else {
                // predecessor is older than the last `recent_count_limit` blocks
                let (tmp_chain, tmp_predecessor_total_work) =
                    self.compute_chain_at_block(predecessor_hash.to_owned());
                predecessor_height = tmp_chain.blocks.len() as u64;
                predecessor_total_work = tmp_predecessor_total_work;
                chain = tmp_chain;
            }
        }
        if chain.blocks[(predecessor_height - 1) as usize].0.difficulty > block.difficulty {
            println!("{{\"error\":\"difficulty must not decrease\"}}");
            return false;
        }

        let timestamp = now();
        if !chain.submit(block.clone(), timestamp) {
            return false;
        }
        self.forks.remove(&predecessor_hash);
        self.forks.insert(
            bhash.to_owned(),
            (
                predecessor_height + 1,
                timestamp,
                predecessor_total_work + u64::pow(16, block.difficulty),
                chain.clone(),
            ),
        );
        self.heads.remove(&Head {
            height: predecessor_height,
            totalWork: predecessor_total_work,
            hash: predecessor_hash,
        });
        self.heads.insert(Head {
            height: predecessor_height + 1,
            totalWork: predecessor_total_work + u64::pow(16, block.difficulty),
            hash: bhash.to_owned(),
        });
        let (main_chain_blocks, main_chain_blocks_set, main_chain_state) = self.get_main_chain();
        self.blocks = main_chain_blocks;
        self.blocks_set = main_chain_blocks_set;
        self.state = main_chain_state;
        // If the block was added to the main chain
        if self.blocks.last().unwrap().0.hash.to_owned() == bhash.to_owned() {
            self.recent_blocks.insert(
                bhash.to_owned(),
                (
                    predecessor_height + 1,
                    timestamp,
                    predecessor_total_work + u64::pow(16, block.difficulty),
                    block,
                    chain.outputs_set,
                    chain.outputs.clone(),
                ),
            );
            if self.recent_blocks_queue.len() == self.recent_count_limit {
                if let Some(v) = self.recent_blocks_queue.pop_front() {
                    self.recent_blocks.remove(&v.to_owned());
                }
            }
            self.recent_blocks_queue.push_back(bhash.to_owned());
        }
        println!("{{\"ok\":[]}}");

        true
    }
    /// Prints the current state of the network.
    ///
    /// Returns `true` if there is a longest chain, otherwise returns `false`.
    pub fn state(&mut self) -> bool {
        if self.heads.len() == 0 {
            println!("{{\"error\":\"must initialize first\"}}");
            return false;
        } else {
            let j = json!({
                "state": {
                    "height": self.state.height,
                    "totalWork": self.state.totalWork,
                    "hash": self.state.hash.to_owned(),
                    "outputs": self.state.outputs
                }
            });
            println!("{}", j.to_string());
        }
        true
    }
    /// Prints a list of all current heads (possible forks) in the network.
    ///
    /// Returns `false` if a genesis block has not yet been initialized, otherwise returns `true`.
    pub fn heads(&mut self) -> bool {
        if self.heads.len() == 0 {
            println!("{{\"error\":\"must initialize first\"}}");
            return false;
        }
        let mut heads = vec![];
        for h in &self.heads {
            heads.push(Head {
                height: h.height,
                totalWork: h.totalWork,
                hash: h.hash.to_owned(),
            });
        }

        let j = json!({ "heads": heads });
        println!("{}", j.to_string());

        true
    }
    /// Prints all the details of the [Network](struct.Network.html).
    pub fn print_details(&mut self) -> bool {
        let _ = self.state();
        let _ = self.heads();
        let j = json!({
            "recent_blocks_queue": self.recent_blocks_queue
        });
        println!("{}", j.to_string());
        let mut rbs = vec![];
        for rb in self.recent_blocks.keys() {
            rbs.push(rb);
        }
        let j2 = json!({ "recent_blocks": rbs });
        println!("{}", j2.to_string());
        let mut forks = vec![];
        for f in self.forks.keys() {
            forks.push(f);
        }
        let j3 = json!({ "forks": forks });
        println!("{}", j3.to_string());

        let mut blocks = vec![];
        for b in &self.blocks {
            blocks.push(b.0.hash.to_owned());
        }
        let j4 = json!({ "blocks": blocks });
        println!("{}", j4.to_string());

        let mut blocks_set = vec![];
        for bs in &self.blocks_set {
            blocks_set.push(bs);
        }
        let j5 = json!({ "blocks_set": blocks_set });
        println!("{}", j5.to_string());

        true
    }
}
