# gRPC client stub for Stargate

## Building
1. Install Rust toolchain:
  
       curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

3. Run build:

       cargo build 

## Running the example

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
       
