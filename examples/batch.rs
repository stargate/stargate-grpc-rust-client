//! Demonstrates sending batches of queries

use connect::*;
use stargate_grpc::*;

mod connect;

const KEYSPACE: &str = "stargate_example_batch";

/// Creates the test keyspace and an empty `users` and `users_by_login` tables
async fn create_schema(client: &mut StargateClient) -> anyhow::Result<()> {
    let builder = QueryBuilder::new().keyspace(KEYSPACE);
    let create_users_table = builder
        .clone()
        .query(
            "CREATE TABLE IF NOT EXISTS users \
            (id bigint primary key, login varchar, emails list<varchar>)",
        )
        .build();
    let create_users_by_login_table = builder
        .clone()
        .query("CREATE TABLE IF NOT EXISTS users_by_login(login varchar primary key, id bigint)")
        .build();

    client.execute_query(create_users_table).await?;
    client.execute_query(create_users_by_login_table).await?;
    Ok(())
}

/// Inserts a user into both tables with a single batch of statements
async fn register_user(client: &mut StargateClient, id: i64, login: &str) -> anyhow::Result<i64> {
    let batch = BatchBuilder::new()
        .keyspace(KEYSPACE)
        .query("INSERT INTO users (id, login, emails) VALUES (:id, :login, :emails)")
        .bind_name("id", id)
        .bind_name("login", login)
        .bind_name("emails", vec![format!("{}@example.net", login)])
        .query("INSERT INTO users_by_login (id, login) VALUES (:id, :login)")
        .bind_name("id", id)
        .bind_name("login", login)
        .build();

    client.execute_batch(batch).await?;
    Ok(id)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = connect().await?;
    println!("Connected");
    create_keyspace(&mut client, KEYSPACE).await?;
    create_schema(&mut client).await?;
    println!("Created schema");
    println!("Inserting data...");
    register_user(&mut client, 1, "user").await?;
    println!("Done");
    Ok(())
}
