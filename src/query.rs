use crate::{
    Consistency, ConsistencyValue, IntoValue, Payload, Query, QueryParameters, Value, Values,
};

/// A wrapper struct allowing us to convert tuples to query values.
/// It is not possible to define a direct conversion from a tuple to a vector
/// because both tuples and vectors are foreign to our crate and trait implementation
/// rules forbid that.
pub struct QueryValues(pub Vec<Value>);

impl From<Vec<Value>> for QueryValues {
    fn from(v: Vec<Value>) -> Self {
        QueryValues(v)
    }
}

/// Builds a [`Query`].
/// Sets the CQL string, binds values and sets query execution parameters.
///
///
/// # Example
/// ```
/// use stargate_grpc::{QueryBuilder, ConsistencyValue, Consistency};
///
/// let query = QueryBuilder::new("SELECT * FROM table WHERE year = :year and month = :month")
///     .named_value("year", 2021)
///     .named_value("month", "October")
///     .keyspace("ks")
///     .consistency(Consistency::LocalQuorum)
///     .build();
/// ```
#[derive(Clone)]
pub struct QueryBuilder {
    cql: String,
    values: Vec<Value>,
    value_names: Vec<String>,
    parameters: QueryParameters,
}

impl QueryBuilder {
    /// Creates a new `QueryBuilder` initialized with a CQL query.
    pub fn new(cql: &str) -> Self {
        QueryBuilder {
            cql: cql.to_string(),
            values: vec![],
            value_names: vec![],
            parameters: Default::default(),
        }
    }

    /// Sets a value at a given index.
    ///
    /// If the internal vector of values is too small, it is automatically resized to
    /// so that the `index` is valid, and any previously
    /// unset values are filled with [`Value::unset()`].
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{QueryBuilder, ConsistencyValue, Consistency};
    ///
    /// let query = QueryBuilder::new("SELECT * FROM table WHERE year = ? and month = ?")
    ///     .value(0, 2021)
    ///     .value(1, "October")
    ///     .build();
    /// ```
    /// # Panics
    /// Will panic if it is called after a call to [`named_value()`](QueryBuilder::named_value)
    pub fn value<T: Into<Value>>(mut self, index: usize, value: T) -> Self {
        if !self.value_names.is_empty() {
            panic!("Mixing named with non-named values is not allowed")
        }
        if index >= self.values.len() {
            self.values.resize(index + 1, Value::unset());
        }
        self.values[index] = value.into_value();
        self
    }

    /// Sets all values at once, from a vector or a value that can
    /// be converted to a vector, e.g. a tuple.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{QueryBuilder, ConsistencyValue, Consistency, Value};
    ///
    /// let cql = "SELECT * FROM table WHERE year = ? and month = ?";
    ///
    /// let query1 = QueryBuilder::new(cql)
    ///     .values((2021, "October"))
    ///     .build();
    ///
    /// let query2 = QueryBuilder::new(cql)
    ///     .values(vec![Value::int(2021), Value::string("October")])
    ///     .build();
    ///
    /// assert_eq!(query1.values, query2.values);
    /// ```
    ///
    /// # Panics
    /// Will panic if it is called after a call to [`named_value()`](QueryBuilder::named_value)
    pub fn values<I: Into<QueryValues>>(mut self, values: I) -> Self {
        if !self.value_names.is_empty() {
            panic!("Mixing named with non-named values is not allowed")
        }
        self.values.extend(values.into().0);
        self
    }

    /// Binds a name to a value.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{QueryBuilder, ConsistencyValue, Consistency};
    ///
    /// let query = QueryBuilder::new("SELECT * FROM table WHERE year = :year and month = :month")
    ///     .named_value("year", 2021)
    ///     .named_value("month", "October")
    ///     .build();
    /// ```
    ///
    /// # Panics
    /// Will panic if mixed with calls to [`value()`](QueryBuilder::value)
    /// or [`values()`](QueryBuilder::values).
    pub fn named_value<T: Into<Value>>(self, name: &str, value: T) -> Self {
        let mut values = self.values;
        let mut value_names = self.value_names;
        if values.len() != value_names.len() {
            panic!("Mixing named with non-named values is not allowed")
        }
        value_names.push(name.to_string());
        values.push(value.into_value());
        QueryBuilder {
            values,
            value_names,
            ..self
        }
    }

    pub fn keyspace(mut self, keyspace: &str) -> Self {
        self.parameters.keyspace = Some(keyspace.to_string());
        self
    }

    pub fn consistency(mut self, consistency: Consistency) -> Self {
        self.parameters.consistency = Some(ConsistencyValue {
            value: consistency.into(),
        });
        self
    }

    pub fn page_size(mut self, size: i32) -> Self {
        self.parameters.page_size = Some(size);
        self
    }

    pub fn paging_state(mut self, paging_state: Vec<u8>) -> Self {
        self.parameters.paging_state = Some(paging_state);
        self
    }

    pub fn tracing(mut self, tracing: bool) -> Self {
        self.parameters.tracing = tracing;
        self
    }

    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.parameters.timestamp = Some(timestamp);
        self
    }

    pub fn serial_consistency(mut self, consistency: ConsistencyValue) -> Self {
        self.parameters.serial_consistency = Some(consistency);
        self
    }

    pub fn parameters(self, parameters: QueryParameters) -> Self {
        QueryBuilder { parameters, ..self }
    }

    pub fn build(self) -> Query {
        use prost::Message;

        let values = if self.values.is_empty() {
            None
        } else {
            let v = Values {
                values: self.values,
                value_names: self.value_names,
            };
            let data = v.encode_to_vec();
            Some(Payload {
                r#type: 0,
                data: Some(prost_types::Any {
                    type_url: "type.googleapis.com/stargate.Values".to_string(),
                    value: data,
                }),
            })
        };

        Query {
            cql: self.cql,
            values,
            parameters: Some(self.parameters),
        }
    }
}
