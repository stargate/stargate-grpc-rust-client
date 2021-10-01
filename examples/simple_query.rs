use std::convert::TryInto;
use std::env;
use std::str::FromStr;

use anyhow::anyhow;

use stargate_grpc::*;

async fn connect() -> anyhow::Result<StargateClient> {
    let args: Vec<_> = std::env::args().collect();
    let default_url = String::from("http://127.0.0.2:8090");
    let url = args.get(1).unwrap_or(&default_url).to_string();
    let token = env::var("SG_TOKEN").map_err(|_| anyhow!("SG_TOKEN not set"))?;
    let token = AuthToken::from_str(token.as_str())?;
    Ok(StargateClient::connect_with_auth(url, token).await?)
}

async fn print_table_contents(client: &mut StargateClient) -> anyhow::Result<()> {
    let query = Query {
        cql: "SELECT id, login, emails FROM test.users".into(),
        values: None,
        parameters: None,
    };

    let response = client.execute_query(query).await?;
    let result_set: ResultSet = response.try_into()?;

    for row in result_set.rows {
        let (id, login, emails): (i64, String, Vec<String>) = row.try_into()?;
        println!("{} {} {:?}", id, login, emails);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = connect().await?;
    print_table_contents(&mut client).await?;
    Ok(())
}
