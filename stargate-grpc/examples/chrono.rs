//! Demonstrates writing and reading chrono dates and timestamps

use std::convert::TryInto;

use chrono::{Date, DateTime, Local};

use connect::config::*;
use connect::*;
use stargate_grpc::*;

#[path = "connect.rs"]
mod connect;

/// Creates the test keyspace and an empty `users` table.
async fn create_schema(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let create_table = QueryBuilder::new()
        .keyspace(keyspace)
        .query(
            r"CREATE TABLE IF NOT EXISTS events (
                sensor bigint,
                day date,
                ts timestamp,
                value varchar,
                PRIMARY KEY ((sensor, day), ts)
            )",
        )
        .build();
    client.execute_query(create_table).await?;
    Ok(())
}

/// Inserts a row with a date and timestamp.
async fn insert_event(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let query = QueryBuilder::new()
        .keyspace(keyspace)
        .query("INSERT INTO events(sensor, day, ts, value) VALUES (?, ?, ?, ?)");

    let ts = Local::now();
    let day = ts.date();
    client
        .execute_query(query.clone().bind((0, day, ts, "event")).build())
        .await?;
    Ok(())
}

/// Fetches some rows with dates and timestamps.
async fn print_events(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let day = Local::now().date();
    let query = QueryBuilder::new()
        .keyspace(keyspace)
        .query("SELECT sensor, day, ts, value FROM events WHERE sensor = ? AND day = ?")
        .bind((0, day))
        .build();

    let result: ResultSet = client.execute_query(query).await?.try_into()?;

    for row in result.rows {
        let (sensor, day, ts, value): (i64, Date<Local>, DateTime<Local>, String) =
            row.try_into()?;
        println!("Event: {}, {}, {}, {}", sensor, day, ts, value);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_args();
    let keyspace = config.keyspace.as_str();
    let mut client = connect(&config).await?;
    println!("Connected");
    create_schema(&mut client, keyspace).await?;
    println!("Created schema");
    insert_event(&mut client, keyspace).await?;
    println!("Inserted data. Now querying.");
    print_events(&mut client, keyspace).await?;
    Ok(())
}
