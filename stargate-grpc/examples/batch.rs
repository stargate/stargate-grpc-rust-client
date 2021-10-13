//! Demonstrates sending batches of queries

use config::*;
use connect::*;
use stargate_grpc::*;

#[path = "connect.rs"]
mod connect;

/// Creates the test keyspace and an empty `users` and `users_by_login` tables
async fn create_schema(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let builder = Query::builder().keyspace(keyspace);
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
async fn register_user(
    client: &mut StargateClient,
    keyspace: &str,
    id: i64,
    login: &str,
) -> anyhow::Result<i64> {
    let batch = Batch::builder()
        .keyspace(keyspace)
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
    let config = Config::from_args();
    let keyspace = config.keyspace.as_str();
    let mut client = connect(&config).await?;
    println!("Connected to {}", config.url);
    create_schema(&mut client, keyspace).await?;
    println!("Created schema");
    println!("Inserting data...");
    register_user(&mut client, keyspace, 1, "user").await?;
    println!("Done");
    Ok(())
}
