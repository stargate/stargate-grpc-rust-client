//! Demonstrates how to connect to Stargate

use stargate_grpc::StargateClient;
use tonic::transport::Endpoint;

use config::Config;
use stargate_grpc::client::default_tls_config;

#[path = "config.rs"]
pub mod config;

/// Connects to Stargate and returns a client that can run queries.
pub async fn connect(config: &Config) -> anyhow::Result<StargateClient> {
    let mut endpoint = Endpoint::new(config.url.clone())?;
    if config.tls {
        endpoint = endpoint.tls_config(default_tls_config())?;
    }
    let channel = endpoint.connect().await?;
    Ok(StargateClient::with_auth(channel, config.token.clone()))
}

#[allow(unused)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_args();
    let mut client = connect(&config).await?;
    println!("Connected to {}", config.url);
    Ok(())
}
