mod ardulink;
mod cli_args;
mod commander;
mod groundlink;
use ardulink::config::ArdulinkConfig;
use clap::Parser;
use log::info;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli_args::Args::parse();
    pretty_env_logger::init();
    info!("Starting conductor with config: {}", args.config);

    let groundlink_server = groundlink::server::start_default_groundlink_server().await?;

    let ardulink_config = ArdulinkConfig{
        connection: ardulink::config::ArdulinkConnectionType::Tcp(String::from("127.0.0.1"), 15760),
    };
    
    let mut ardulink = ardulink::connection::ArdulinkConnection::new(ardulink_config.connection)?;
    ardulink.start_task().await?;

    let _ = tokio::try_join!(groundlink_server);

    Ok(())
}
