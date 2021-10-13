//! Demonstrates how to store Rust structs as UDT values in Cassandra and how to get them back

use stargate_grpc::*;
use std::convert::TryInto;

use config::*;
use connect::*;

#[path = "connect.rs"]
mod connect;

#[derive(Debug, IntoValue, TryFromValue)]
struct Address {
    street: String,
    number: i64,
    #[stargate(default)]
    apartment: Option<i64>,
}

async fn create_schema(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let create_type = Query::builder()
        .keyspace(keyspace)
        .query(
            "CREATE TYPE IF NOT EXISTS address(\
                    street VARCHAR, \
                    number BIGINT, \
                    apartment BIGINT)",
        )
        .build();
    let create_table = Query::builder()
        .keyspace(keyspace)
        .query(
            "CREATE TABLE IF NOT EXISTS users_with_addr(\
                    id BIGINT PRIMARY KEY, \
                    addresses LIST<FROZEN<address>>)",
        )
        .build();

    client.execute_query(create_type).await?;
    client.execute_query(create_table).await?;
    Ok(())
}

async fn insert_data(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let insert = Query::builder()
        .keyspace(keyspace)
        .query("INSERT INTO users_with_addr(id, addresses) VALUES (:id, :addr)")
        .bind_name("id", 1)
        .bind_name(
            "addr",
            vec![
                Address {
                    street: "Long St".to_string(),
                    number: 7870,
                    apartment: Some(13),
                },
                Address {
                    street: "Nice St".to_string(),
                    number: 12,
                    apartment: None,
                },
            ],
        )
        .build();

    client.execute_query(insert).await?;
    Ok(())
}

async fn print_all_users(client: &mut StargateClient, keyspace: &str) -> anyhow::Result<()> {
    let select = Query::builder()
        .keyspace(keyspace)
        .query("SELECT id, addresses FROM users_with_addr")
        .build();

    let result: ResultSet = client.execute_query(select).await?.try_into()?;
    for row in result.rows {
        let (id, addresses): (i64, Vec<Address>) = row.try_into()?;
        println!("{}, {:?}", id, addresses);
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
    insert_data(&mut client, keyspace).await?;
    println!("Inserted");
    print_all_users(&mut client, keyspace).await?;
    println!("Done");
    Ok(())
}
