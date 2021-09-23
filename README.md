# gRPC client stub for Stargate

## Building
1. Install Rust toolchain:
  
       curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

3. Run build:

       cargo build 

## Running the example

1. Set up Stargate server 
2. Create keyspace `test` with default settings.

       CREATE KEYSPACE test WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

3. Create a table `test.foo` and insert some data:

       CREATE TABLE test.foo(pk bigint primary key, value varchar);
       INSERT INTO test.foo(pk, value) values (1, 'foo');
       INSERT INTO test.foo(pk, value) values (2, 'bar'); 

5. Fetch the authentication token and store it in the `SG_TOKEN` environment variable:

       curl -L -X POST 'http://127.0.0.2:8081/v1/auth' \
            -H 'Content-Type: application/json' \
            --data-raw '{
               "username": "cassandra",
               "password": "cassandra"
            }'
              
       {"authToken":"2df7e75d-92aa-4cda-9816-f96ccbc91d80"}
 
       export SG_TOKEN=2df7e75d-92aa-4cda-9816-f96ccbc91d80

6. Run the example:


        cargo run --example simple_query

        Connected to Stargate
        Received response: Response { metadata: MetadataMap { headers: {"content-type": "application/grpc", "grpc-encoding": "identity", "grpc-accept-encoding": "gzip", "grpc-status": "0"} }, message: Response { warnings: [], traces: None, result: Some(ResultSet(Payload { r#type: Cql, data: Some(Any { type_url: "type.googleapis.com/stargate.ResultSet", value: [10, 8, 10, 2, 8, 2, 18, 2, 112, 107, 10, 11, 10, 2, 8, 13, 18, 5, 118, 97, 108, 117, 101, 18, 11, 10, 2, 24, 4, 10, 5, 58, 3, 98, 97, 114, 18, 11, 10, 2, 24, 2, 10, 5, 58, 3, 102, 111, 111] }) })) }, extensions: Extensions }
        Got data: Any { type_url: "type.googleapis.com/stargate.ResultSet", value: [10, 8, 10, 2, 8, 2, 18, 2, 112, 107, 10, 11, 10, 2, 8, 13, 18, 5, 118, 97, 108, 117, 101, 18, 11, 10, 2, 24, 4, 10, 5, 58, 3, 98, 97, 114, 18, 11, 10, 2, 24, 2, 10, 5, 58, 3, 102, 111, 111] }

