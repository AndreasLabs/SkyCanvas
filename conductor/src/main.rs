mod ardulink;
mod cli_args;
mod commander;
mod groundlink;
use clap::Parser;
use log::info;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli_args::Args::parse();
    pretty_env_logger::init();
    info!("Starting conductor with config: {}", args.config);
    Ok(())
}
