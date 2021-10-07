use rustyline::error::ReadlineError;
use rustyline::Editor;
use serde_json::{Result, Value};

use mycoinlib::{Block, InitGenesis, Network, SubmittedBlock};

fn init_chain(d: &str, network: &mut Network) -> Result<()> {
    let b: InitGenesis = match serde_json::from_str(d) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };
    let mut block = Block::new(
        b.init.difficulty,
        b.init.hash,
        b.init.nonce,
        b.init.predecessor,
        b.init.transactions,
    );
    if block.validate() {
        let _ = network.init(block);
    }
    Ok(())
}

fn submit_block(d: &str, network: &mut Network) -> Result<()> {
    let b: SubmittedBlock = match serde_json::from_str(d) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };
    let mut block = Block::new(
        b.block.difficulty,
        b.block.hash,
        b.block.nonce,
        b.block.predecessor,
        b.block.transactions,
    );
    if block.validate() {
        let _ = network.submit(block);
    }
    Ok(())
}

fn handle_commands(data: &str, network: &mut Network) -> Result<()> {
    let val: Value = serde_json::from_str(data)?;
    if let Some(_) = val.get("init") {
        let icv = init_chain(data, network);
        if icv.is_ok() {
            return icv;
        }
    }
    if let Some(field) = val.get("query") {
        if field == "state" {
            let _ = network.state();
        }
        if field == "heads" {
            let _ = network.heads();
        }
        if field == "print" {
            let _ = network.print_details();
        }
    }
    if let Some(_) = val.get("block") {
        let sbv = submit_block(data, network);
        if sbv.is_ok() {
            return sbv;
        }
    }

    Ok(())
}

fn main() {
    let mut network = Network::new(2);

    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history("history.txt");
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let _ = handle_commands(line.as_str(), &mut network);
                println!();
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
