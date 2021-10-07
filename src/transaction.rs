use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A struct that represent an input or output of a [transaction](struct.Transaction.html).
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Output {
    pub id: u64,
    pub amount: u64,
}

/// A transaction contains any number of inputs and any number of outputs, which
/// must sum to the same amount.
#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub inputs: Vec<Output>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    pub fn input_value(&self) -> u64 {
        self.inputs.iter().map(|input| input.amount).sum()
    }
    pub fn output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.amount).sum()
    }
    pub fn inputs(&self) -> HashSet<Output> {
        let mut hn = HashSet::new();
        for i in &self.inputs {
            hn.insert(Output {
                id: i.id,
                amount: i.amount,
            });
        }
        hn
    }
    pub fn outputs(&self) -> HashSet<Output> {
        let mut hn = HashSet::new();
        for i in &self.outputs {
            hn.insert(Output {
                id: i.id,
                amount: i.amount,
            });
        }
        hn
    }
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 0
    }
}
