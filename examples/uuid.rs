//! Demonstrates setting and reading `uuid:Uuid` values

use anyhow::anyhow;
use std::convert::TryInto;
use std::env;
use std::str::FromStr;
use uuid::Uuid;

use stargate_grpc::*;

const KEYSPACE: &str = "stargate_example_uuid";

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

/// Connects to Stargate and returns a client that can run queries.
async fn connect() -> anyhow::Result<StargateClient> {
    let url = get_url();
    let token = get_auth_token()?;
    Ok(StargateClient::connect_with_auth(url, token).await?)
}

/// Creates the test keyspace and an empty `users` table.
async fn create_schema(client: &mut StargateClient) -> anyhow::Result<()> {
    let create_keyspace = QueryBuilder::new()
        .query(
            format!(
                "CREATE KEYSPACE IF NOT EXISTS {} \
                    WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}",
                KEYSPACE
            )
            .as_str(),
        )
        .build();

    let create_table = QueryBuilder::new()
        .keyspace(KEYSPACE)
        .query("CREATE TABLE IF NOT EXISTS users (id uuid primary key, name varchar)")
        .build();

    client.execute_query(create_keyspace).await?;
    client.execute_query(create_table).await?;
    Ok(())
}

/// Inserts a user with given name and a randomly generated unique identifier.
async fn register_user(client: &mut StargateClient, name: &str) -> anyhow::Result<Uuid> {
    let uuid = Uuid::new_v4();
    let query = QueryBuilder::new()
        .keyspace(KEYSPACE)
        .query("INSERT INTO users (id, name) VALUES (:id, :name)")
        .bind_name("id", uuid)
        .bind_name("name", name)
        .build();
    client.execute_query(query).await?;
    Ok(uuid)
}

/// Looks up a user by id and returns it as
async fn fetch_user(
    client: &mut StargateClient,
    id: Uuid,
) -> anyhow::Result<Option<(Uuid, String)>> {
    let query = QueryBuilder::new()
        .keyspace(KEYSPACE)
        .query("SELECT id, name FROM users WHERE id = :id")
        .bind_name("id", id)
        .build();
    let result: ResultSet = client.execute_query(query).await?.try_into()?;
    match result.rows.into_iter().next() {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = connect().await?;
    println!("Connected");
    create_schema(&mut client).await?;
    println!("Created schema");
    println!("Inserting data...");
    let id = register_user(&mut client, "user").await?;
    println!("Querying...");
    if let Some((id, name)) = fetch_user(&mut client, id).await? {
        println!("Fetched row: ({}, {})", id, name)
    }
    println!("Done");
    Ok(())
}
