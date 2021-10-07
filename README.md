# mycoin

A PoW blockchain client.

rustdoc: https://laughing-bell-6f7a59.netlify.app/mycoinlib

## Getting Started

### Compile the package

```sh
cargo build
```

### Run the client

```sh
cargo run
```

### Build documentation and view in browser

```sh
cargo doc --no-deps --release --open
```

## High level workflow

We are primarily using two data structures: `Network` and `Blockchain`. `Network` keeps track of all the possible forks, maintains the chain state and a list of all blocks in the main chain (longest chain, whichever one has the highest PoW). Each instance of `Blockchain` essentially acts as a single chain (or fork) in a network.
