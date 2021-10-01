use std::convert::TryInto;
use std::env;
use std::str::FromStr;

use anyhow::anyhow;

use stargate_grpc::*;

/// Connects to Stargate and returns a client that can run queries
async fn connect() -> anyhow::Result<StargateClient> {
    let args: Vec<_> = std::env::args().collect();
    let default_url = String::from("http://127.0.0.2:8090");
    let url = args.get(1).unwrap_or(&default_url).to_string();
    let token = env::var("SG_TOKEN").map_err(|_| anyhow!("SG_TOKEN not set"))?;
    let token = AuthToken::from_str(token.as_str())?;
    Ok(StargateClient::connect_with_auth(url, token).await?)
}

/// Creates the test keyspace and an empty `users` table
async fn create_schema(client: &mut StargateClient) -> anyhow::Result<()> {
    let create_keyspace = QueryBuilder::new(
        "CREATE KEYSPACE IF NOT EXISTS test \
        WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}",
    )
    .build();
    let create_table = QueryBuilder::new(
        "CREATE TABLE IF NOT EXISTS users\
        (id bigint primary key, login varchar, emails list<varchar>)",
    )
    .keyspace("test")
    .build();

    client.execute_query(create_keyspace).await?;
    client.execute_query(create_table).await?;
    Ok(())
}

/// Inserts some sample data into the `users` table
async fn insert_data(client: &mut StargateClient) -> anyhow::Result<()> {
    let insert =
        QueryBuilder::new("INSERT INTO users(id, login, emails) VALUES (?, ?, ?)").keyspace("test");

    for id in 0..10 {
        let login = format!("user_{}", id);
        let emails = vec![
            format!("{}@example.net", login),
            format!("{}@mail.example.net", login),
        ];
        let query = insert.clone().values((id, login, emails)).build();
        client.execute_query(query).await?;
    }
    Ok(())
}

/// Fetches all rows from the `user` table.
async fn select_all(client: &mut StargateClient) -> anyhow::Result<ResultSet> {
    let query = QueryBuilder::new("SELECT id, login, emails FROM test.users").build();
    let result = client.execute_query(query).await?.try_into()?;
    Ok(result)
}

/// Fetches one row from the table. Demonstrates how to use named values.
async fn select_one(client: &mut StargateClient, id: i32) -> anyhow::Result<ResultSet> {
    let query = QueryBuilder::new("SELECT id, login, emails FROM test.users WHERE id = :id")
        .named_value("id", id)
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
