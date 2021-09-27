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

       CREATE TABLE test.users(id bigint primary key, login varchar, emails list<varchar>);
       INSERT INTO test.users(id, login, emails) values (1, 'one', ['one@example.net']);
       INSERT INTO test.users(id, login, emails) values (2, 'two', ['2@example.net', 'two@example.net']); 

4. Fetch the authentication token and store it in the `SG_TOKEN` environment variable:

       curl -L -X POST 'http://127.0.0.2:8081/v1/auth' \
            -H 'Content-Type: application/json' \
            --data-raw '{
               "username": "cassandra",
               "password": "cassandra"
            }'
              
       {"authToken":"2df7e75d-92aa-4cda-9816-f96ccbc91d80"}
 
       export SG_TOKEN=2df7e75d-92aa-4cda-9816-f96ccbc91d80

5. Run the example:

       cargo run --example simple_query http://127.0.0.2:8090
       2 two ["2@example.net", "two@example.net"]
       1 one ["one@example.net"]
    
