# Auxilliary

Rust Arbitrage CLI.

Data from Pyth <https://pyth.network/developers/accounts/?cluster=mainnet-beta>#

```text
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src
│   ├── error.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── price_engine.rs
│   └── state.rs
└── target
    └── debug
        └── auxiliary
```

For more information, see the program's help menu, `auxiliary --help`.

## Build

```Rust
// Development -> product in /target/debug/
cargo build // --features devnet
cargo run -- --payer ~/.config/solana/id.json arbitrage --tickrate 800 // ms

// Release -> product in /target/release/
cargo build --release --no-default-features --features mainnet 
```

## Running

Running `/target/release/` with no argument prints the program's help menu.
There are a few arguments needed for all subcommands, which can also be passed as environment variables.
Additionally, the project uses dotenv as well, so it's recommended to copy .env.example to .env and configure it appropriately, to avoid having to pass arguments every time.

For additional log, change the log level in your `.env` file:

```env
# Optional
RUST_LOG=auxiliary=info // debug, trace...
```

## Rust/Solana env setup

VsCode <https://code.visualstudio.com/> (use rust-analyzer extension instead of default rls)

Install rustup <https://rustup.rs/>

Install Solana <https://docs.solana.com/cli/install-solana-cli-tools>

(not needed yet)
Install Anchor <https://project-serum.github.io/anchor/getting-started/installation.html#install-solana>
