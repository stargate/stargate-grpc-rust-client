//! Demonstrates how to connect to Stargate

use config::Config;
use stargate_grpc::StargateClient;

#[path = "config.rs"]
pub mod config;

/// Connects to Stargate and returns a client that can run queries.
pub async fn connect(config: &Config) -> anyhow::Result<StargateClient> {
    Ok(StargateClient::builder()
        .uri(config.url.as_str())?
        .auth_token(config.token.clone())
        .tls(config.tls_config()?)
        .connect()
        .await?)
}

#[allow(unused)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_args();
    let mut client = connect(&config).await?;
    println!("Connected to {}", config.url);
    Ok(())
}
