mod commands;

use anchor_client::Cluster;
use clap::Parser;
use solana_sdk::signature::{read_keypair_file, Keypair};
use std::rc::Rc;
use std::str::FromStr;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "devnet")]
    cluster: String,

    #[arg(long)]
    keypair: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let cluster = Cluster::from_str(&args.cluster)?;
    let payer = match args.keypair {
        Some(path) => read_keypair_file(&path).expect("invalid keypair file"),
        None => Keypair::new(),
    };
    commands::run_demo(cluster, Rc::new(payer))?;
    Ok(())
}
