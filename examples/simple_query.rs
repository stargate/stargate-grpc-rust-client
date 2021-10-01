use std::convert::TryInto;

use anyhow::anyhow;
use tonic::metadata::AsciiMetadataValue;

use stargate_grpc::stargate_client::*;
use stargate_grpc::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let default_url = "http://127.0.0.2:8090";
    let url = args.get(1).map(|s| s.as_str()).unwrap_or(default_url);

    let token =
        std::env::var("SG_TOKEN").map_err(|_| anyhow!("Missing SG_TOKEN environment variable"))?;
    let token = AuthToken::from_str(token.as_str())?;

    let mut client = StargateClient::connect_with_auth(url.to_owned(), token).await?;

    let query = Query {
        cql: "SELECT id, login, emails FROM test.users".into(),
        values: None,
        parameters: None,
    };

    let response = client.execute_query(query).await?;
    let result_set: ResultSet = response.try_into()?;

    for row in result_set.rows {
        let mut values = row.values.into_iter();
        let id: i64 = values
            .next()
            .ok_or(anyhow!("Missing column: id"))?
            .try_into()?;
        let login: String = values
            .next()
            .ok_or(anyhow!("Missing column: login"))?
            .try_into()?;
        let emails: Vec<String> = values
            .next()
            .ok_or(anyhow!("Missing column: emails"))?
            .try_into()?;
        println!("{} {} {:?}", id, login, emails);
    }
    Ok(())
}
