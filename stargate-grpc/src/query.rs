//! Utilities for building queries.

use crate::into_value::IntoValue;
use crate::proto::{
    Batch, BatchParameters, BatchQuery, Consistency, Payload, Query, QueryParameters, Value, Values,
};

impl From<Vec<Value>> for Values {
    fn from(v: Vec<Value>) -> Self {
        Values {
            value_names: vec![],
            values: v,
        }
    }
}

/// Builds a [`Query`].
/// Sets the CQL string, binds values and sets query execution parameters.
///
/// # Example
/// ```
/// use stargate_grpc::{Query, Consistency};
///
/// let query = Query::builder()
///     .keyspace("ks")
///     .consistency(Consistency::LocalQuorum)
///     .query("SELECT * FROM table WHERE year = :year and month = :month")
///     .bind_name("year", 2021)
///     .bind_name("month", "October")
///     .build();
/// ```
///
/// A single `QueryBuilder` can be used to create more than one query, because it is cloneable.
/// You can set the default query parameters at the beginning, and then clone it multiple times
/// to set a different query string or different query arguments each time:
///
/// # Example
/// ```
/// use stargate_grpc::{Query, Consistency};
///
/// let query_defaults = Query::builder()
///     .keyspace("ks")
///     .consistency(Consistency::LocalQuorum);
///
/// let query1 = query_defaults.clone().query("SELECT * FROM table1").build();
/// let query2 = query_defaults.clone().query("SELECT * FROM table2").build();
/// ```
///
#[derive(Default, Clone)]
pub struct QueryBuilder {
    cql: Option<String>,
    payload: PayloadBuilder,
    parameters: QueryParameters,
}

impl QueryBuilder {
    /// Creates a new `QueryBuilder` with default parameters and no query.
    ///
    /// You have to call [`query`](QueryBuilder::query) to set the CQL query string
    /// before calling [`build`](QueryBuilder::build).
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the CQL query string.
    pub fn query(mut self, cql: &str) -> Self {
        self.cql = Some(cql.to_string());
        self
    }

    /// Sets all values at once, from a vector or a value that can
    /// be converted to a vector, e.g. a tuple.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{Query, Value};
    ///
    /// let cql = "SELECT * FROM table WHERE year = ? and month = ?";
    ///
    /// let query1 = Query::builder()
    ///     .query(cql)
    ///     .bind((2021, "October"))
    ///     .build();
    ///
    /// let query2 = Query::builder()
    ///     .query(cql)
    ///     .bind(vec![Value::bigint(2021), Value::string("October")])
    ///     .build();
    ///
    /// assert_eq!(query1.values, query2.values);
    /// ```
    ///
    /// # Panics
    /// Will panic if it is called after a call to [`bind_name`](QueryBuilder::bind_name)
    pub fn bind<I: Into<Values>>(mut self, values: I) -> Self {
        self.payload.bind(values);
        self
    }

    /// Sets a value at a given index.
    ///
    /// If the internal vector of values is too small, it is automatically resized to
    /// so that the `index` is valid, and any previously
    /// unset values are filled with [`Value::unset`].
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::Query;
    ///
    /// let query = Query::builder()
    ///     .query("SELECT * FROM table WHERE year = ? and month = ?")
    ///     .bind_ith(0, 2021)
    ///     .bind_ith(1, "October")
    ///     .build();
    /// ```
    /// # Panics
    /// Will panic if it is called after a call to [`bind_name`](QueryBuilder::bind_name)
    pub fn bind_ith<T: Into<Value>>(mut self, index: usize, value: T) -> Self {
        self.payload.bind_ith(index, value);
        self
    }

    /// Binds a name to a value.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::Query;
    ///
    /// let query = Query::builder()
    ///     .query("SELECT * FROM table WHERE year = :year and month = :month")
    ///     .bind_name("year", 2021)
    ///     .bind_name("month", "October")
    ///     .build();
    /// ```
    ///
    /// # Panics
    /// Will panic if mixed with calls to [`bind`](QueryBuilder::bind)
    /// or [`bind_ith`](QueryBuilder::bind_ith).
    pub fn bind_name<T: Into<Value>>(mut self, name: &str, value: T) -> Self {
        self.payload.bind_name(name, value);
        self
    }

    /// Sets the keyspace the query will apply to.
    ///
    /// See [`QueryParameters::keyspace`].
    pub fn keyspace(mut self, keyspace: &str) -> Self {
        self.parameters.keyspace = Some(keyspace.to_string());
        self
    }

    /// Sets the consistency level of the query.
    /// # Example
    /// ```
    /// use stargate_grpc::{Consistency, Query};
    ///
    /// let query = Query::builder()
    ///     .query("SELECT * FROM table")
    ///     .consistency(Consistency::One);
    /// ```
    /// See [`QueryParameters::consistency`].
    pub fn consistency(mut self, consistency: Consistency) -> Self {
        self.parameters.consistency = Some(crate::proto::ConsistencyValue {
            value: consistency.into(),
        });
        self
    }

    /// Sets the serial consistency level (if the query is a lightweight transaction).
    ///
    /// See [`QueryParameters::serial_consistency`].
    pub fn serial_consistency(mut self, consistency: Consistency) -> Self {
        self.parameters.serial_consistency = Some(crate::proto::ConsistencyValue {
            value: consistency.into(),
        });
        self
    }

    /// Sets the maximum number of rows that will be returned in the response.
    ///
    /// See [`QueryParameters::page_size`].
    pub fn page_size(mut self, size: i32) -> Self {
        self.parameters.page_size = Some(size);
        self
    }

    /// Sets a paging state that indicates where to resume iteration in the result set.
    ///
    /// See [`QueryParameters::paging_state`].
    pub fn paging_state(mut self, paging_state: Vec<u8>) -> Self {
        self.parameters.paging_state = Some(paging_state);
        self
    }

    /// Sets whether the server should collect tracing information about the execution of the query.
    ///
    /// See [`QueryParameters::tracing`].
    pub fn tracing(mut self, tracing: bool) -> Self {
        self.parameters.tracing = tracing;
        self
    }

    /// Sets the query timestamp (in microseconds).
    ///
    /// See [`QueryParameters::timestamp`].
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.parameters.timestamp = Some(timestamp);
        self
    }

    /// Sets all parameters of the query at once.
    ///
    /// Overwrites any parameters that were set before.
    pub fn parameters(self, parameters: QueryParameters) -> Self {
        QueryBuilder { parameters, ..self }
    }

    /// Builds the query that can be passed to
    /// [`StargateClient::execute_query`](crate::StargateClient::execute_query).
    ///
    /// # Panics
    /// Will panic if the query string was not set.
    pub fn build(mut self) -> Query {
        Query {
            cql: self.cql.expect("cql string"),
            values: self.payload.build(),
            parameters: Some(self.parameters),
        }
    }
}

impl Query {
    /// Returns a fresh builder for building a query
    pub fn builder() -> QueryBuilder {
        QueryBuilder::new()
    }
}

/// Builds a batch of queries.
///
/// # Example
/// ```
/// use stargate_grpc::{Batch, Consistency};
///
/// let batch = Batch::builder()
///     .keyspace("example")
///     .consistency(Consistency::LocalQuorum)
///     .query("INSERT INTO users (id, login, email) VALUES (?, ?, ?)")
///     .bind((0, "admin", "admin@example.net"))
///     .query("INSERT INTO users_by_login (id, login) VALUES (?, ?)")
///     .bind((0, "admin"))
///     .build();
/// ```
#[derive(Default, Clone)]
pub struct BatchBuilder {
    cql: Option<String>,
    payload: PayloadBuilder,
    parameters: BatchParameters,
    built_queries: Vec<BatchQuery>,
}

impl BatchBuilder {
    /// Creates a new `BatchBuilder` with no queries in it.
    pub fn new() -> BatchBuilder {
        Default::default()
    }

    /// Adds a CQL query to the batch.
    ///
    /// If the query has arguments, set their values with
    /// one of the `bind` functions.
    pub fn query(mut self, cql: &str) -> Self {
        self.finalize_query();
        self.cql = Some(cql.to_string());
        self
    }

    /// Binds all arguments of the lately added query at once,
    /// from a vector or a value that can be converted to a vector, e.g. a tuple.
    ///
    /// # Panics
    /// Will panic if it is called after a call to [`bind_name`](BatchBuilder::bind_name)
    pub fn bind<I: Into<Values>>(mut self, values: I) -> Self {
        self.payload.bind(values);
        self
    }

    /// Binds an argument of the recently added query at a given index.
    ///
    /// This function can be called multiple times, to bind several arguments.
    /// If the internal vector of values is too small, it is automatically resized to
    /// so that the `index` is valid, and any previously
    /// unset values are filled with [`Value::unset`].
    pub fn bind_ith<T: Into<Value>>(mut self, index: usize, value: T) -> Self {
        self.payload.bind_ith(index, value);
        self
    }

    /// Binds a name to a value.
    ///
    /// This function can be called multiple times, to bind several arguments.
    ///
    /// # Panics
    /// Will panic if mixed with calls to [`bind`](BatchBuilder::bind)
    /// or [`bind_ith`](BatchBuilder::bind_ith).
    pub fn bind_name<T: Into<Value>>(mut self, name: &str, value: T) -> Self {
        self.payload.bind_name(name, value);
        self
    }

    /// Sets the keyspace every query in the batch will apply to.
    ///
    /// See [`BatchParameters::keyspace`].
    pub fn keyspace(mut self, keyspace: &str) -> Self {
        self.parameters.keyspace = Some(keyspace.to_string());
        self
    }

    /// Sets the consistency level of all queries in the batch.
    ///
    /// See [`BatchParameters::consistency`].
    pub fn consistency(mut self, consistency: Consistency) -> Self {
        self.parameters.consistency = Some(crate::proto::ConsistencyValue {
            value: consistency.into(),
        });
        self
    }

    /// Sets whether the server should collect tracing information about the execution of the batch.
    ///
    /// See [`BatchParameters::tracing`].
    pub fn tracing(mut self, tracing: bool) -> Self {
        self.parameters.tracing = tracing;
        self
    }

    /// Sets the query timestamp (in microseconds).
    ///
    /// See [`BatchParameters::timestamp`].
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.parameters.timestamp = Some(timestamp);
        self
    }

    /// Sets the serial consistency level (if the query is a lightweight transaction).
    ///
    /// See [`BatchParameters::serial_consistency`].
    pub fn serial_consistency(mut self, consistency: Consistency) -> Self {
        self.parameters.serial_consistency = Some(crate::proto::ConsistencyValue {
            value: consistency.into(),
        });
        self
    }

    /// Sets all parameters of the batch at once.
    ///
    /// Overwrites any parameters that were set before.
    pub fn parameters(mut self, parameters: BatchParameters) -> Self {
        self.parameters = parameters;
        self
    }

    /// Finalizes building and returns the `Batch` that can be passed to
    /// [`StargateClient::execute_batch`](crate::StargateClient::execute_batch).
    pub fn build(mut self) -> Batch {
        self.finalize_query();
        Batch {
            r#type: 0,
            queries: self.built_queries,
            parameters: Some(self.parameters),
        }
    }

    fn finalize_query(&mut self) {
        if let Some(cql) = self.cql.take() {
            self.built_queries.push(BatchQuery {
                cql,
                values: self.payload.build(),
            });
        }
    }
}

impl Batch {
    /// Returns a fresh builder for building a batch of queries
    pub fn builder() -> BatchBuilder {
        BatchBuilder::new()
    }
}

/// The logic of building the query payload shared between [`QueryBuilder`] and [`BatchBuilder`]
#[derive(Default, Clone)]
struct PayloadBuilder {
    values: Vec<Value>,
    value_names: Vec<String>,
}

impl PayloadBuilder {
    pub fn bind<I: Into<Values>>(&mut self, values: I) {
        if !self.value_names.is_empty() {
            panic!("Mixing named with non-named values is not allowed")
        }
        let values = values.into();
        self.values.extend(values.values);
        self.value_names.extend(values.value_names)
    }

    pub fn bind_ith<T: Into<Value>>(&mut self, index: usize, value: T) {
        if !self.value_names.is_empty() {
            panic!("Mixing named with non-named values is not allowed")
        }
        if index >= self.values.len() {
            self.values.resize(index + 1, Value::unset());
        }
        self.values[index] = value.into_value();
    }

    pub fn bind_name<T: Into<Value>>(&mut self, name: &str, value: T) {
        if self.values.len() != self.value_names.len() {
            panic!("Mixing named with non-named values is not allowed")
        }
        self.value_names.push(name.to_string());
        self.values.push(value.into_value());
    }

    pub fn build(&mut self) -> Option<Payload> {
        use prost::Message;

        if self.values.is_empty() {
            None
        } else {
            let v = Values {
                values: self.values.drain(0..).collect(),
                value_names: self.value_names.drain(0..).collect(),
            };
            let data = v.encode_to_vec();
            Some(Payload {
                r#type: 0,
                data: Some(prost_types::Any {
                    type_url: "type.googleapis.com/stargate.Values".to_string(),
                    value: data,
                }),
            })
        }
    }
}
