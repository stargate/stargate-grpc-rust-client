//! Demonstrates writing and reading `uuid:Uuid` values

use std::convert::TryInto;
use uuid::Uuid;

use connect::*;
use stargate_grpc::*;

mod connect;

const KEYSPACE: &str = "stargate_example_uuid";

/// Creates the test keyspace and an empty `users` table.
async fn create_schema(client: &mut StargateClient) -> anyhow::Result<()> {
    let create_table = QueryBuilder::new()
        .keyspace(KEYSPACE)
        .query("CREATE TABLE IF NOT EXISTS users (id uuid primary key, name varchar)")
        .build();

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
    create_keyspace(&mut client, KEYSPACE).await?;
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
