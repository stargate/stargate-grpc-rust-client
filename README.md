# Rust Client Driver for Stargate Using gRPC 

This crate provides a high-level async Rust driver for querying [DataStax Stargate](https://stargate.io/).
It exposes the client stubs generated from gRPC proto files together with a set of 
utilities that make them easier to work with.

## Quick start guide
Add required dependencies. You'll need at least `stargate-grpc` and an async framework, 
e.g. tokio:

```toml
[devependencies]
stargate-grpc = { git = "https://github.com/stargate/stargate-grpc-rust-client" }
tokio = { version = "1", features = ["full"]}
```

Add the following line to the includes in the source code of your app:
```rust
use stargate_grpc::*;
```

### Establishing the connection
The main structure that provides the interface to Stargate is `StargateClient`. 
Pass the Stargate endpoint URL and the authentication token to `connect_with_auth()` to obtain 
an instance:

```rust
let token = "00000000-0000-0000-0000-000000000000";  // substitute with a real authentication token 
let url = "http://localhost:8090";                   // substitute with a real Stargate URL
let token = AuthToken::from_str(token)?;
let mut client = StargateClient::connect_with_auth(url, token).await?;
```

### Querying 
Use `QueryBuilder` to create a query, bind query values and pass query parameters:

```rust
let query = QueryBuilder::new("SELECT login, emails FROM users WHERE id = :id")
    .keyspace("test")                           // set the keyspace the query applies to
    .consistency(Consistency::LocalQuorum)      // set consistency level
    .named_value("id", 1000)                    // bind :id to 1000
    .build();                                   // build the Query
```

Run the query and wait for its results:
```rust
let response = client.execute_query(query).await?;  // send the query and wait for gRPC response
let result_set: ResultSet = response.try_into()?;   // convert the response into ResultSet
```

### Processing the result set
The result set comes back as a collection of rows. A`Row` can be easily unpacked
into a tuple:

```rust
for row in result_set.rows {
    let (login, emails): (String, Vec<String>) = row.try_into()?;
    // ...
}
```

It is also possible to read each field separately and convert it to desired type, without
dropping the original `row`:
```rust
for row in result_set.rows {
    let login: String = row.get(0)?;
    let emails: Vec<String> = row.get(1)?;
    // ...
}
```


## Building from source
1. Install Rust toolchain:
  
       curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

2. Run `build` in the root directory of the project:

       git clone https://github.com/stargate/stargate-grpc-rust-client stargate-grpc
       cd stargate-grpc
       cargo build

## Running the example
For your convenience, the project sources contain an example that demonstrates
connecting, creating schema, inserting data and querying. You'll need a working instance
of a Stargate cluster to be able to run it. Refer to the 
[official Stargate documentation](https://stargate.io/docs/stargate/1.0/developers-guide/install/install_overview.html)
for more details on how to setup Stargate.

1. Set up Stargate server. Start Cassandra cluster and launch Stargate:

       ccm create stargate -v 3.11.8 -n 1 -s -b
       ./starctl --cluster-name stargate --cluster-seed 127.0.0.1 --cluster-version 3.11 --listen 127.0.0.2 \
                 --bind-to-listen-address --simple-snitch

3. Fetch the authentication token and store it in the `SG_TOKEN` environment variable:

       curl -L -X POST 'http://127.0.0.2:8081/v1/auth' \
            -H 'Content-Type: application/json' \
            --data-raw '{
               "username": "cassandra",
               "password": "cassandra"
            }'
              
       {"authToken":"2df7e75d-92aa-4cda-9816-f96ccbc91d80"}
 
       export SG_TOKEN=2df7e75d-92aa-4cda-9816-f96ccbc91d80

4. Run the example:

       cargo run --example basic 
       Finished dev [unoptimized + debuginfo] target(s) in 0.04s
       Running `target/debug/examples/basic`
       Connected
       Created schema
       Inserted data. Now querying.
       All rows:
       2 user_2 ["user_2@example.net", "user_2@mail.example.net"]
       3 user_3 ["user_3@example.net", "user_3@mail.example.net"]
       7 user_7 ["user_7@example.net", "user_7@mail.example.net"]
       9 user_9 ["user_9@example.net", "user_9@mail.example.net"]
       4 user_4 ["user_4@example.net", "user_4@mail.example.net"]
       0 user_0 ["user_0@example.net", "user_0@mail.example.net"]
       8 user_8 ["user_8@example.net", "user_8@mail.example.net"]
       5 user_5 ["user_5@example.net", "user_5@mail.example.net"]
       6 user_6 ["user_6@example.net", "user_6@mail.example.net"]
       1 user_1 ["user_1@example.net", "user_1@mail.example.net"]
       Row with id = 1:
       1 user_1 ["user_1@example.net", "user_1@mail.example.net"]
       
