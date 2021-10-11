//! Demonstrates writing and reading `uuid:Uuid` values

use std::convert::TryInto;
use uuid::Uuid;

use connect::config::*;
use connect::*;
use stargate_grpc::*;

#[path = "connect.rs"]
mod connect;

async fn create_schema(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let create_table = QueryBuilder::new()
        .keyspace(keyspace)
        .query("CREATE TABLE IF NOT EXISTS users_uuid (id uuid primary key, name varchar)")
        .build();

    client.execute_query(create_table).await?;
    Ok(())
}

/// Inserts a user with given name and a randomly generated unique identifier.
async fn register_user(
    client: &mut StargateClient,
    keyspace: &str,
    name: &str,
) -> anyhow::Result<Uuid> {
    let uuid = Uuid::new_v4();
    let query = QueryBuilder::new()
        .keyspace(keyspace)
        .query("INSERT INTO users_uuid (id, name) VALUES (:id, :name)")
        .bind_name("id", uuid)
        .bind_name("name", name)
        .build();
    client.execute_query(query).await?;
    Ok(uuid)
}

/// Looks up a user by uuid
async fn fetch_user(
    client: &mut StargateClient,
    keyspace: &str,
    id: Uuid,
) -> anyhow::Result<Option<(Uuid, String)>> {
    let query = QueryBuilder::new()
        .keyspace(keyspace)
        .query("SELECT id, name FROM users_uuid WHERE id = :id")
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
    let config = Config::from_args();
    let keyspace = config.keyspace.as_str();
    let mut client = connect(&config).await?;
    println!("Connected");
    create_schema(&mut client, keyspace).await?;
    println!("Created schema");
    println!("Inserting data...");
    let id = register_user(&mut client, keyspace, "user").await?;
    println!("Querying...");
    if let Some((id, name)) = fetch_user(&mut client, keyspace, id).await? {
        println!("Fetched row: ({}, {})", id, name)
    }
    println!("Done");
    Ok(())
}
