//! # Rust gRPC Client Driver for Stargate
//!
//! This crate provides a high-level async Rust driver
//! for querying [Stargate](https://stargate.io/).
//!
//! ## Features
//! - All of the Stargate gRPC protocol messages exposed as Rust structures and enums
//! - Token-based authentication
//! - Asynchronous querying
//! - Query builder with easy binding of variables by names or positions
//! - Optional compile-time type-checking of query bind values
//! - Easy conversions between gRPC value types and common Rust types; support for
//!   primitive types, lists, maps, tuples and user-defined-types, with arbitrary nesting levels
//! - Result set paging
//!
//! ## Usage
//! Add required dependencies.
//!
//! ```toml
//! [devependencies]
//! stargate-grpc = "0.1.0"
//! tokio = { version = "1", features = ["full"]}
//! ```
//!
//!
//! ### Establishing the connection
//! The main structure that provides the interface to Stargate is [`StargateClient`].
//! The simplest way to obtain an instance is to use the provided
//! [`builder`](StargateClient::builder):
//!
//! ```
//! use std::str::FromStr;
//! use stargate_grpc::client::{default_tls_config, AuthToken, StargateClient};
//!
//! # async fn connect() -> anyhow::Result<()>{
//! let uri = "http://localhost:8090/";                    // Stargate URI
//! let token = "00000000-0000-0000-0000-000000000000";    // Stargate authentication token
//! let token = AuthToken::from_str(token)?;
//! let mut client = StargateClient::builder()
//!     .uri(uri)?
//!     .auth_token(token)
//!     .tls(Some(default_tls_config()?))                  // optionally to enable TLS
//!     .connect()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! If you want to control the properties of the connection which are not exposed by the builder,
//! add [`tonic`](https://docs.rs/tonic/0.5.2/tonic/) to the dependencies of the project and create
//! the connection manually. Then use [`StargateClient::with_auth`] to wrap the connection and
//! the authentication token:
//!
//! ```
//! # use std::str::FromStr;
//! use std::time::Duration;
//! # use stargate_grpc::client::{default_tls_config, AuthToken, StargateClient};
//! #
//! # async fn connect() -> anyhow::Result<()>{
//! # let uri = "http://localhost:8090";
//! # let token = "00000000-0000-0000-0000-000000000000";
//! # let token = AuthToken::from_str(token).unwrap();
//! let channel = tonic::transport::Endpoint::new(uri)?
//!     .connect_timeout(Duration::from_secs(30))
//!     .tcp_nodelay(true)
//!     .connect().await?;
//! let mut client = StargateClient::with_auth(channel, token);
//! # Ok(())
//! # }
//! ```
//!
//! ### Querying
//! Use a [`QueryBuilder`] to create a query, bind query arguments and pass query parameters:
//!
//! ```rust
//! use stargate_grpc::{Consistency, Query};
//! let query = Query::builder()
//!     .keyspace("test")                              // set the keyspace the query applies to
//!     .consistency(Consistency::LocalQuorum)         // set consistency level
//!     .query("SELECT * FROM users WHERE id = :id")   // set CQL query text (required)
//!     .bind_name("id", 1000)                         // bind :id to 1000
//!     .build();                                      // build the Query
//! ```
//!
//! Run the query and wait for its results:
//! ```rust
//! # use std::convert::TryInto;
//! # use stargate_grpc::{StargateClient, Query};
//! # async fn run_query(client: &mut StargateClient, query: Query) -> anyhow::Result<()> {
//! use stargate_grpc::ResultSet;
//! let response = client.execute_query(query).await?; // send the query and wait for gRPC response
//! let result_set: ResultSet = response.try_into()?;  // convert the response into ResultSet
//! # Ok(())
//! # }
//! ```
//!
//! If you need to send more than one query in a single request, use a [`BatchBuilder`].
//! All queries in the batch will share the same parameters, such as
//! keyspace, consistency level or timestamp. Send the batch for execution with
//! [`StargateClient::execute_batch`].
//!
//! ### Processing the result set
//! A [`ResultSet`] comes back as a collection of rows. A [`Row`] can be easily unpacked
//! into a tuple:
//
//! ```rust
//! # use std::convert::TryInto;
//! # use stargate_grpc::ResultSet;
//! # fn process_results(result_set: ResultSet) -> anyhow::Result<()> {
//! for row in result_set.rows {
//!     let (login, emails): (String, Vec<String>) = row.try_into()?;
//!     // ...
//! }
//! # Ok(())
//! # }
//! ```
//!
//! It is also possible to read each field separately and convert it to desired type, without
//! dropping the original `row`:
//! ```rust
//! # use std::convert::TryInto;
//! # use stargate_grpc::ResultSet;
//! # fn process_results(result_set: ResultSet) -> anyhow::Result<()> {
//! for row in result_set.rows {
//!     let login: String = row.get(0)?;
//!     let emails: Vec<String> = row.get(1)?;
//!     // ...
//! }
//! # Ok(())
//! # }
//! ```
//!
//!
//!
//! ## Representation of values
//!
//! The values bound in queries and the values received in the `Row`s of a `ResultSet`
//! are internally represented by `struct` [`Value`]. A `Value` wraps an `enum` that can
//! hold one of many data types. `Value` provides factory functions that produce values
//! of desired type, so it is easy to construct them.
//!
//! ```rust
//! use stargate_grpc::Value;
//!
//! let bool = Value::boolean(true);
//! let int = Value::int(1);
//! let double = Value::double(1.0);
//! let string = Value::string("stargate");
//! let list = Value::list(vec![Value::int(1), Value::int(2), Value::int(3)]);
//! let map = Value::map(vec![("key1", Value::int(1)), ("key2", Value::int(2))]);
//! ```
//!
//! List or maps can hold values of different types:
//! ```rust
//! # use stargate_grpc::Value;
//! let heterogeneous_list = vec![Value::int(1), Value::double(3.14)];
//! ```
//!
//! Values can be used in [`QueryBuilder::bind`] or [`QueryBuilder::bind_name`]:
//!
//! ```rust
//! use stargate_grpc::{Query, Value};
//! let query = Query::builder()
//!     .query("SELECT login, emails FROM users WHERE id = :id")
//!     .bind_name("id", Value::int(1000))
//!     .build();
//! ```
//!
//! A [`Row`] is represented by a vector of `Value`s:
//! ```rust
//! use std::convert::TryInto;
//! use stargate_grpc::{Row, Value};
//!
//! let row = Row { values: vec![Value::int(1), Value::double(3.14)] };
//! ```
//!
//! Values can be converted to and from other commonly used Rust types.
//! For more examples, refer to the documentation of modules [`from_value`] and [`into_value`].
//!
//! ### Working with UUIDs
//! This crate provides only a very lightweight representation of UUIDs: [`proto::Uuid`].
//! A UUID is internally represented as an vector of bytes.
//! That struct does not provide any functions to generate nor manipulate the
//! UUID value, however, it should be fairly easy to convert to from other UUID representations.
//!
//! To get support for conversions from and to
//! [`uuid::UUID`](https://docs.rs/uuid/0.8/uuid/struct.Uuid.html),
//! bring [`uuid`](https://crates.io/crates/uuid) on the dependency list and enable feature `uuid`.
//! ```toml
//! [dependencies]
//! uuid = "0.8
//! stargate-grpc = { version = "0.1", features = ["uuid"] }
//! ```
//!
//! ### Working with times, dates and timestamps
//! This crate doesn't define its own fully-fledged
//! structures for representing dates, times and timestamps.
//! It allows an easy integration with external structures instead.
//!
//! A time value is internally represented as an `u64` number of nanoseconds elapsed
//! since midnight.
//! A date is internally represented as an `u32` number of days elapsed
//! since Unix epoch. Dates before Unix epoch are not representable.
//! A timestamp is internally represented as an `i64` number of milliseconds
//! elapsed since Unix epoch. Timestamps can be negative.
//! You can convert integer types directly to and from time, date and timestamp values.
//!
//! To get automatic conversions to and from
//! [`chrono::Date`](https://docs.rs/chrono/0.4/chrono/struct.Date.html) and
//! [`chrono::DateTime`](https://docs.rs/chrono/0.4/chrono/struct.DateTime.html)
//! include [`chrono`](https://crates.io/crates/chrono) on the dependency list and
//! enable feature `chrono`.
//!
//! ```toml
//! [dependencies]
//! chrono = "0.4"
//! stargate-grpc = { version = "0.1", features = ["chrono"] }
//! ```
//!
//! ### Mapping Rust structs to user defined types
//! Feature [`stargate-grpc-derive`](stargate_grpc_derive) (enabled by default) allows to
//! generate conversions between `Value`s and your Rust structs by adding
//! the `#[derive(IntoValue, TryFromValue)]` attribute on top of a struct definition.
//!
//!

pub use client::{AuthToken, StargateClient};
pub use from_value::TryFromValue;
pub use into_value::{DefaultGrpcType, IntoValue};
pub use proto::{Batch, Consistency, Query, ResultSet, Row, Value};
#[cfg(feature = "stargate-grpc-derive")]
pub use stargate_grpc_derive::*;

pub mod client;
pub mod from_value;
pub mod into_value;
pub mod query;
pub mod result;

pub mod error;
pub mod types;

/// Structures automatically generated from gRPC protocol definition files located in `api/`.
pub mod proto {
    tonic::include_proto!("stargate");
}

/// Holds a key and a value pair; used in map representation.
///
/// Maps are passed as collections of key-value pairs, where items (0, 2, 4, ...) are keys,
/// and items (1, 3, 5, ...) are values. This means key-value pairs are not encoded as nested
/// collections. Hence, in order to receive a map, we must convert it to `Vec<KeyValue<K, V>>`
/// and *not* into `Vec<(K, V)>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue<K, V>(pub K, pub V);

impl<K, V> KeyValue<K, V> {
    pub fn into_tuple(self) -> (K, V) {
        (self.0, self.1)
    }
}
