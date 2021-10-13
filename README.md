# Rust gRPC Client Driver for Stargate  

This crate provides a high-level async Rust driver for querying [Stargate](https://stargate.io/).
It exposes the client stubs generated from gRPC proto files together with a set of 
utilities that make them easier to work with.

- [Features](#features)
- [Quick start guide](#quick-start-guide)
   - [Connecting](#establishing-the-connection)
   - [Querying](#querying)
   - [Processing the result set](#processing-the-result-set)
- [Building](#building-from-source)
- [Running the examples](#running-the-examples)

## Features
- All of the Stargate gRPC protocol messages exposed as Rust structures and enums
- Token-based authentication
- Asynchronous querying
- Query builder with easy binding of variables by names or positions 
- Optional compile-time type-checking of query bind values
- Easy conversions between gRPC value types and common Rust types; support for
  primitive types, lists, maps, tuples and user-defined-types, with arbitrary nesting levels
- Result set paging

## Quick start guide
Add required dependencies. You'll need at least `stargate-grpc` and an async framework, 
e.g. [tokio](https://tokio.rs/) or [async-std](https://async.rs/). 

_Caution! Currently this repo is private and the project hasn't been published on crates.io. 
Until that happens, you need to point Cargo to this repository using the git SSH link._

```toml
[dependencies]
stargate-grpc = { git = "ssh://git@github.com/stargate/stargate-grpc-rust-client.git" }
tokio = { version = "1", features = ["full"]}
```

_In order to allow Cargo to authenticate with GitHub keys, put this in `~/.cargo/config`_:
```toml
[net]
git-fetch-with-cli = true
```

At this point you should be able to build the project now with `cargo build` and it would fetch and compile 
the dependencies. 

For convenience, add the following line to the includes in the source code of your app:
```rust
use stargate_grpc::*;
```

### Connecting
The main structure that provides the interface to Stargate is `StargateClient`.
The simplest way to obtain an instance is to use the provided `builder`:

```rust
let mut client = StargateClient::builder()
    .uri("http://localhost:8090/")?
    .auth_token(AuthToken::from_str("00000000-0000-0000-0000-000000000000")?)                                         
    .tls(Some(client::default_tls_config()?))   // optional
    .connect()
    .await?;
```

### Querying 
Use `QueryBuilder` to create a query, bind query values and pass query parameters:

```rust
let query = QueryBuilder::new()
    .keyspace("test")                           // set the keyspace the query applies to
    .consistency(Consistency::LocalQuorum)      // set consistency level
    .query("SELECT login, emails FROM users WHERE id = :id")
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

## Running the examples
For your convenience, this project contains a bunch of examples located in the `examples` directory, 
which demonstrate connecting, creating schemas, inserting data and querying. You'll need a working instance
of a Stargate cluster to be able to run it. Refer to the 
[official Stargate documentation](https://stargate.io/docs/stargate/1.0/developers-guide/install/install_overview.html)
for more details on how to setup Stargate.

Each example program accepts an URL of the stargate coordinator, 
the authentication token and the keyspace name:

    cargo run --example <example> [-- [--keyspace <keyspace>] [--token <auth token>] [--tls] [<url>]] 

The authentication token value can be also given in the `SG_TOKEN` environment variable.

1. Set up Stargate server. Start Cassandra cluster and launch Stargate:

       ccm create stargate -v 3.11.8 -n 1 -s -b
       ./starctl --cluster-name stargate --cluster-seed 127.0.0.1 --cluster-version 3.11 --listen 127.0.0.2 \
                 --bind-to-listen-address --simple-snitch

2. Obtain the authentication token:

       curl -L -X POST 'http://127.0.0.2:8081/v1/auth' \
            -H 'Content-Type: application/json' \
            --data-raw '{
               "username": "cassandra",
               "password": "cassandra"
            }'
              
       {"authToken":"2df7e75d-92aa-4cda-9816-f96ccbc91d80"}

3. Set the authentication token variable:
 
       export SG_TOKEN=2df7e75d-92aa-4cda-9816-f96ccbc91d80

4. Run the `keyspace` example to test the connection and create the test keyspace (default keyspace name: `stargate_examples`)

       cargo run --example keyspace 
       Connected to http://127.0.0.2:8090
       Created keyspace stargate_examples

5. Run the other examples:

       cargo run --example query 
       Finished dev [unoptimized + debuginfo] target(s) in 0.04s
       Running `target/debug/examples/basic`
       Connected to http://127.0.0.2:8090
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


