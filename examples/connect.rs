//! Demonstrates how to connect and create a keyspace

use std::env;
use std::str::FromStr;

use anyhow::anyhow;

use stargate_grpc::{AuthToken, QueryBuilder, StargateClient};

/// Returns the URL of the Stargate coordinator we need to connect to.
fn get_url() -> String {
    let args: Vec<_> = std::env::args().collect();
    let default_url = String::from("http://127.0.0.2:8090");
    args.get(1).unwrap_or(&default_url).to_string()
}

/// Returns the authentication token read from the `SG_TOKEN` environment variable.
fn get_auth_token() -> anyhow::Result<AuthToken> {
    let token = env::var("SG_TOKEN").map_err(|_| anyhow!("SG_TOKEN not set"))?;
    Ok(AuthToken::from_str(token.as_str())?)
}

/// Connects to Stargate, prepares the keyspace and returns a client that can run queries.
pub async fn connect() -> anyhow::Result<StargateClient> {
    let url = get_url();
    let token = get_auth_token()?;
    Ok(StargateClient::connect_with_auth(url, token).await?)
}

/// Creates a test keyspace with default settings.
pub async fn create_keyspace(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let cql = format!(
        "CREATE KEYSPACE IF NOT EXISTS {} \
            WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}",
        keyspace
    );
    let create_keyspace = QueryBuilder::new().query(cql.as_str()).build();
    client.execute_query(create_keyspace).await?;
    Ok(())
}

#[allow(unused)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = connect().await?;
    println!("Connected");
    create_keyspace(&mut client, "stargate_example_connect").await?;
    println!("Created the keyspace");
    Ok(())
}