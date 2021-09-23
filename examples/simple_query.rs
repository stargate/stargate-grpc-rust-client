use std::process::exit;

use tonic::metadata::AsciiMetadataValue;

use stargate_grpc::stargate_client::*;
use stargate_grpc::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = match std::env::var("SG_TOKEN") {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Authentication token SG_TOKEN not set or invalid: {}", e);
            exit(1);
        }
    };

    let mut client = StargateClient::connect("http://127.0.0.2:8090").await?;
    eprintln!("Connected to Stargate");

    let query = Query {
        cql: "SELECT * FROM test.foo".into(),
        values: None,
        parameters: None,
    };

    let mut request = tonic::Request::new(query);
    request.metadata_mut().insert(
        "x-cassandra-token",
        AsciiMetadataValue::from_str(token.as_str())?,
    );

    let response = client.execute_query(request).await?;
    eprintln!("Received response: {:?}", &response);

    if let stargate_grpc::response::Result::ResultSet(payload) =
        response.get_ref().result.as_ref().unwrap()
    {
        // TODO: Unpack data
        let data = payload.data.as_ref().unwrap();
        println!("Got data: {:?}", data);
    }
    Ok(())
}
