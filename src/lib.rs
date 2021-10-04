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
//! stargate-grpc = { git = "https://github.com/stargate/stargate-grpc-rust-client" }
//! tokio = { version = "1", features = ["full"]}
//! ```
//!
//!
//! ### Establishing the connection
//! The main structure that provides the interface to Stargate is [`StargateClient`].
//! Pass the Stargate endpoint URL and the authentication token to
//! [`StargateClient::connect_with_auth()`] to obtain an instance:
//!
//! ```rust
//! use std::str::FromStr;
//! use stargate_grpc::{AuthToken, StargateClient};
//!
//! # async fn connect() -> anyhow::Result<()>{
//! let token = "00000000-0000-0000-0000-000000000000";  // substitute with an authentication token
//! let url = "http://localhost:8090";                   // substitute with a Stargate URL
//! let token = AuthToken::from_str(token).unwrap();
//! let mut client = StargateClient::connect_with_auth(url, token).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Querying
//! Use `QueryBuilder` to create a query, bind query values and pass query parameters:
//!
//! ```rust
//! use stargate_grpc::{Consistency, QueryBuilder};
//!
//! let query = QueryBuilder::new("SELECT login, emails FROM users WHERE id = :id")
//!     .keyspace("test")                           // set the keyspace the query applies to
//!     .consistency(Consistency::LocalQuorum)      // set consistency level
//!     .named_value("id", 1000)                    // bind :id to 1000
//!     .build();                                   // build the Query
//! ```
//!
//! Run the query and wait for its results:
//! ```rust
//! # use std::convert::TryInto;
//! # use stargate_grpc::{Query, StargateClient};
//!
//! # async fn run_query(client: &mut StargateClient, query: Query) -> anyhow::Result<()> {
//!
//! use stargate_grpc::ResultSet;
//! let response = client.execute_query(query).await?;  // send the query and wait for gRPC response
//! let result_set: ResultSet = response.try_into()?;   // convert the response into ResultSet
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### Processing the result set
//! The result set comes back as a collection of rows. A`Row` can be easily unpacked
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
//! Values can be used in [`QueryBuilder::value`] or [`QueryBuilder::named_value`]:
//!
//! ```rust
//! use stargate_grpc::{QueryBuilder, Value};
//! let query = QueryBuilder::new("SELECT login, emails FROM users WHERE id = :id")
//!     .named_value("id", Value::int(1000))
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
//! Refer to the documentation of modules [`from_value`] and [`into_value`].
//!


use std::fmt::{Debug, Display, Formatter};

use prost::DecodeError;

pub use client::*;
pub use from_value::*;
pub use into_value::*;
pub use query::*;
pub use result::*;

pub mod client;
pub mod from_value;
pub mod into_value;
pub mod query;
pub mod result;

pub mod types;

tonic::include_proto!("stargate");

/// Error thrown when some data received from the wire could not be properly
/// converted to a desired Rust type.
#[derive(Clone, Debug)]
pub struct ConversionError {
    /// Describes the reason why the conversion failed.
    pub kind: ConversionErrorKind,
    /// Debug string of the source value that failed to be converted.
    pub source: String,
    /// Name of the target Rust type that the value failed to convert to.
    pub target_type_name: String,
}

#[derive(Clone, Debug)]
pub enum ConversionErrorKind {
    /// When the converter didn't know how to convert one type to another
    /// because the conversion hasn't been defined.
    Incompatible,

    /// When the number of elements in a vector or a tuple
    /// does not match the expected number of elements.
    WrongNumberOfItems { actual: usize, expected: usize },

    /// When the converter attempted to decode a binary blob,
    /// but the conversion failed due to invalid data.
    GrpcDecodeError(DecodeError),
}

impl ConversionError {
    fn new<S: Debug, T>(kind: ConversionErrorKind, source: S) -> ConversionError {
        ConversionError {
            kind,
            source: format!("{:?}", source),
            target_type_name: std::any::type_name::<T>().to_string(),
        }
    }

    fn incompatible<S: Debug, T>(source: S) -> ConversionError {
        Self::new::<S, T>(ConversionErrorKind::Incompatible, source)
    }

    fn wrong_number_of_items<S: Debug, T>(
        source: S,
        actual: usize,
        expected: usize,
    ) -> ConversionError {
        Self::new::<S, T>(
            ConversionErrorKind::WrongNumberOfItems { actual, expected },
            source,
        )
    }

    fn decode_error<S: Debug, T>(source: S, error: DecodeError) -> ConversionError {
        Self::new::<S, T>(ConversionErrorKind::GrpcDecodeError(error), source)
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot convert value {} to {}",
            self.source, self.target_type_name
        )
    }
}
