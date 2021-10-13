//! Demonstrates connecting, creating schema, inserting data and querying

use std::convert::TryInto;

use config::*;
use connect::*;
use stargate_grpc::*;

#[path = "connect.rs"]
mod connect;

/// Creates an empty `users` table
async fn create_schema(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let create_table = Query::builder()
        .keyspace(keyspace)
        .query(
            "CREATE TABLE IF NOT EXISTS users \
                (id bigint primary key, login varchar, emails list<varchar>)",
        )
        .build();

    client.execute_query(create_table).await?;
    Ok(())
}

/// Inserts some sample data into the `users` table
async fn insert_data(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let insert = Query::builder()
        .keyspace(keyspace)
        .query("INSERT INTO users(id, login, emails) VALUES (?, ?, ?)");

    for id in 0..10 {
        let login = format!("user_{}", id);
        let emails = vec![
            format!("{}@example.net", login),
            format!("{}@mail.example.net", login),
        ];
        let query = insert.clone().bind((id, login, emails)).build();
        client.execute_query(query).await?;
    }
    Ok(())
}

/// Fetches all rows from the `user` table.
async fn select_all(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<ResultSet> {
    let query = Query::builder()
        .keyspace(keyspace)
        .query("SELECT id, login, emails FROM users")
        .build();
    let result = client.execute_query(query).await?.try_into()?;
    Ok(result)
}

/// Fetches one row from the table. Demonstrates how to use named values.
async fn select_one(
    client: &mut StargateClient,
    keyspace: &str,
    id: i32,
) -> anyhow::Result<ResultSet> {
    let query = Query::builder()
        .keyspace(keyspace)
        .query("SELECT id, login, emails FROM users WHERE id = :id")
        .bind_name("id", id)
        .build();
    let result = client.execute_query(query).await?.try_into()?;
    Ok(result)
}

/// Prints the result set to the standard output.
/// May return error if the row values fail to convert to expected types.
fn print_rows(result_set: ResultSet) -> anyhow::Result<()> {
    for row in result_set.rows {
        let (id, login, emails): (i64, String, Vec<String>) = row.try_into()?;
        println!("{} {} {:?}", id, login, emails);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_args();
    let keyspace = config.keyspace.as_str();
    let mut client = connect(&config).await?;
    println!("Connected to {}", config.url);
    create_schema(&mut client, keyspace).await?;
    println!("Created schema");
    insert_data(&mut client, keyspace).await?;
    println!("Inserted data. Now querying.");
    println!("All rows:");
    print_rows(select_all(&mut client, keyspace).await?)?;
    println!("Row with id = 1:");
    print_rows(select_one(&mut client, keyspace, 1).await?)?;
    Ok(())
}
