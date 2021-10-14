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
//! Call [`Query::builder`] to set a CQL string, bind query arguments
//! set query parameters and finally produce a `Query`:
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
//! If you need to send more than one query in a single request, create a [`Batch`].
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
//! It is also possible to read each field separately and convert it to desired type:
//! ```rust
//! # use std::convert::TryInto;
//! # use stargate_grpc::ResultSet;
//! # fn process_results(result_set: ResultSet) -> anyhow::Result<()> {
//! for mut row in result_set.rows {
//!     let login: String = row.try_take(0)?;
//!     let emails: Vec<String> = row.try_take(1)?;
//!     // ...
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Alternativelly, you can convert a whole `Row` into a struct with a mapper obtained
//! from [`ResultSet::mapper`](ResultSet::mapper):
//!
//! ```
//! # #[cfg(feature = "macros")]
//! # {
//! use stargate_grpc::{ResultSet, TryFromRow};
//! # fn process_results(result_set: ResultSet) -> anyhow::Result<()> {
//!
//! #[derive(TryFromRow)]
//! struct User {
//!     login: String,
//!     emails: Vec<String>
//! }
//!
//! let mapper = result_set.mapper()?;
//! for row in result_set.rows {
//!     let user: User = mapper.try_unpack(row)?;
//!     // ...
//! }
//! # Ok(())
//! # }}
//! ```
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
//! let int = Value::bigint(1);
//! let double = Value::double(1.0);
//! let string = Value::string("stargate");
//! let list = Value::list(vec![Value::bigint(1), Value::bigint(2), Value::bigint(3)]);
//! let map = Value::map(vec![("key1", Value::bigint(1)), ("key2", Value::bigint(2))]);
//! ```
//!
//! List or maps can hold values of different types:
//! ```rust
//! # use stargate_grpc::Value;
//! let heterogeneous_list = vec![Value::bigint(1), Value::double(3.14)];
//! ```
//!
//! Values can be used in calls to `bind` or `bind_name` used when building queries or batches:
//!
//! ```rust
//! use stargate_grpc::{Query, Value};
//! let query = Query::builder()
//!     .query("SELECT login, emails FROM users WHERE id = :id")
//!     .bind_name("id", Value::bigint(1000))
//!     .build();
//! ```
//!
//! A [`Row`] is represented by a vector of `Value`s:
//! ```rust
//! use std::convert::TryInto;
//! use stargate_grpc::{Row, Value};
//!
//! let row = Row { values: vec![Value::bigint(1), Value::double(3.14)] };
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
//! since midnight. Hence, `Value::time(0)` denotes midnight.
//!
//! A date is internally represented as an `u32` number where value 2^31 denotes Unix Epoch.
//! For convenience and compatibility with most other date representations, values are convertible
//! to and from `i32` type where 0 denotes the Unix Epoch. Therefore the Unix Epoch can be simply
//! written as `Value::date(0)` which is equivalent to `Value::raw_date(1 << 31)`.
//!
//! A timestamp is internally represented as an `i64` number of milliseconds
//! elapsed since Unix epoch. Timestamps can be negative.
//!
//! Using integers to represent dates is error-prone, therefore
//! this library comes with conversions
//! from higher-level structures like [`SystemTime`](std::time::SystemTime):
//!
//! ```rust
//! use std::time::SystemTime;
//! use stargate_grpc::Value;
//!
//! let unix_epoch_1 = Value::timestamp(SystemTime::UNIX_EPOCH);
//! let unix_epoch_2 = Value::timestamp(0);
//! assert_eq!(unix_epoch_1, unix_epoch_2);
//! ```
//!
//! More time related features are provided by an optional `chrono` feature.
//! If you enable `chrono` feature, you get conversions for
//! [`chrono::Date`](https://docs.rs/chrono/0.4/chrono/struct.Date.html) and
//! [`chrono::DateTime`](https://docs.rs/chrono/0.4/chrono/struct.DateTime.html).
//!
//! ```toml
//! [dependencies]
//! chrono = "0.4"
//! stargate-grpc = { version = "0.1", features = ["chrono"] }
//! ```
//!
//! ### Mapping Rust structs to user defined types
//! Feature [`stargate-grpc-derive`](../stargate_grpc_derive) allows to
//! generate conversions between `Value`s and your Rust structs by adding
//! the `#[derive(IntoValue, TryFromValue)]` attribute on top of a struct definition.
//!
//!

pub use client::{AuthToken, StargateClient};
pub use from_value::TryFromValue;
pub use into_value::{DefaultCqlType, IntoValue};
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
