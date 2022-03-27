use anchor_client::{solana_sdk::signer::keypair, Cluster};
use auxiliary as lib;
use clap::{Parser, Subcommand};
use lib::Product;
use std::{env, num::ParseIntError, time::Duration};

#[derive(Parser)]
#[clap(term_width = 80)]
struct Cli {
    /// RPC endpoint.
    #[clap(short, long, env = "SOLANA_RPC_URL")]
    rpc_url: String,

    /// Websocket endpoint.
    #[clap(long, env = "SOLANA_WS_URL")]
    ws_url: String,

    /// Path to keypair. If not set, the JSON encoded keypair is read
    /// from $SOLANA_PAYER_KEY instead.
    #[clap(short, long)]
    payer: Option<std::path::PathBuf>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Do stuff
    PriceFetch {
        /// Interval for oracle price checks, in milliseconds
        #[clap(long, default_value = "400", parse(try_from_str = parse_milliseconds))]
        interval: Duration,
    },
}

fn main() -> Result<(), lib::Error> {
    dotenv::dotenv().ok();

    {
        use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            // https://no-color.org/
            .with_ansi(env::var_os("NO_COLOR").is_none())
            .finish()
            .init();
    }

    let Cli {
        rpc_url,
        ws_url,
        payer,
        command,
    } = Cli::parse();

    let payer = match payer {
        Some(p) => keypair::read_keypair_file(&p).unwrap_or_else(|_| {
            panic!("Failed to read keypair from {}", p.to_string_lossy())
        }),
        None => match env::var("SOLANA_PAYER_KEY").ok() {
            Some(k) => keypair::read_keypair(&mut k.as_bytes())
                .expect("Failed to parse $SOLANA_PAYER_KEY"),
            None => panic!("Could not load payer key,"),
        },
    };

    let cluster = Cluster::Custom(rpc_url, ws_url);

    let app_state: &'static _ = Box::leak(Box::new(lib::AppState::new(
        cluster,
        payer,
        // The product we will support for operations
        vec![Product::SolUsd],
    )));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    match command {
        Command::PriceFetch { interval } => {
            rt.block_on(lib::price_engine::run(
                app_state,
                lib::price_engine::PriceFetchConfig { interval },
            ))?;
        }
    };

    Ok(())
}

fn parse_milliseconds(s: &str) -> Result<Duration, ParseIntError> {
    <u64 as std::str::FromStr>::from_str(s).map(Duration::from_millis)
}
