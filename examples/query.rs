//! Demonstrates connecting, creating schema, inserting data and querying

use std::convert::TryInto;

use connect::*;
use stargate_grpc::*;

mod connect;

const KEYSPACE: &str = "stargate_example_query";

/// Creates an empty `users` table
async fn create_schema(client: &mut StargateClient) -> anyhow::Result<()> {
    let create_table = QueryBuilder::new()
        .keyspace(KEYSPACE)
        .query(
            "CREATE TABLE IF NOT EXISTS users \
                (id bigint primary key, login varchar, emails list<varchar>)",
        )
        .build();

    client.execute_query(create_table).await?;
    Ok(())
}

/// Inserts some sample data into the `users` table
async fn insert_data(client: &mut StargateClient) -> anyhow::Result<()> {
    let insert = QueryBuilder::new()
        .keyspace(KEYSPACE)
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
async fn select_all(client: &mut StargateClient) -> anyhow::Result<ResultSet> {
    let query = QueryBuilder::new()
        .keyspace(KEYSPACE)
        .query("SELECT id, login, emails FROM users")
        .build();
    let result = client.execute_query(query).await?.try_into()?;
    Ok(result)
}

/// Fetches one row from the table. Demonstrates how to use named values.
async fn select_one(client: &mut StargateClient, id: i32) -> anyhow::Result<ResultSet> {
    let query = QueryBuilder::new()
        .keyspace(KEYSPACE)
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
    let mut client = connect().await?;
    println!("Connected");
    create_keyspace(&mut client, KEYSPACE).await?;
    create_schema(&mut client).await?;
    println!("Created schema");
    insert_data(&mut client).await?;
    println!("Inserted data. Now querying.");
    println!("All rows:");
    print_rows(select_all(&mut client).await?)?;
    println!("Row with id = 1:");
    print_rows(select_one(&mut client, 1).await?)?;
    Ok(())
}
