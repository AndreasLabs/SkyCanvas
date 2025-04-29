mod ardulink;
mod cli_args;
mod commander;

mod redis;
mod state;

use ardulink::config::ArdulinkConfig;
use redis::RedisOptions;
use clap::Parser;
use log::info;

use anyhow::Result;
use state::State;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli_args::Args::parse();
    pretty_env_logger::init();
    info!("Starting conductor with config: {}", args.config);


    let ardulink_config = ArdulinkConfig{
        connection: ardulink::config::ArdulinkConnectionType::Tcp(String::from("127.0.0.1"), 5760),
    };
    let redis_options = RedisOptions{
        host: "127.0.0.1".to_string(),
        port: Some(6379),
        password: None,
    };
    let state = State::new(redis_options);
    let mut ardulink = ardulink::connection::ArdulinkConnection::new(ardulink_config.connection, &state)?;
    ardulink.start_task().await?;

    Ok(())
}
