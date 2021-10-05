use anyhow::anyhow;
use std::env;
use std::str::FromStr;

use stargate_grpc::*;

const KEYSPACE: &str = "stargate_example_batch";

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

/// Connects to Stargate and returns a client that can run queries
async fn connect() -> anyhow::Result<StargateClient> {
    let url = get_url();
    let token = get_auth_token()?;
    Ok(StargateClient::connect_with_auth(url, token).await?)
}

/// Creates the test keyspace and an empty `users` and `users_by_login` tables
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

    client.execute_query(create_keyspace).await?;
    client.execute_query(create_users_table).await?;
    client.execute_query(create_users_by_login_table).await?;
    Ok(())
}

/// Inserts a user into both tables with a single batch of statements
async fn register_user(
    client: &mut StargateClient,
    login: &str,
    id: &mut i64,
) -> anyhow::Result<i64> {
    *id += 1;
    let id = *id;

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
    create_schema(&mut client).await?;
    println!("Created schema");
    println!("Inserting data...");
    let mut id = 0;
    register_user(&mut client, "user", &mut id).await?;
    println!("Done");
    Ok(())
}
