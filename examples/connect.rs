//! Demonstrates how to connect to Stargate

use stargate_grpc::StargateClient;

use config::Config;

#[path = "config.rs"]
pub mod config;

/// Connects to Stargate and returns a client that can run queries.
pub async fn connect(config: &Config) -> anyhow::Result<StargateClient> {
    Ok(StargateClient::connect_with_auth(config.url.clone(), config.token.clone()).await?)
}

#[allow(unused)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_args();
    let mut client = connect(&config).await?;
    println!("Connected to {}", config.url);
    Ok(())
}
