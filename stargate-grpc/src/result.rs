//! Automatic data type conversions and utilities useful for processing query results.

use crate::error::ConversionError;
use crate::from_value::TryFromValue;
use crate::proto::{Response, ResultSet, Row};
use std::collections::HashMap;

use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

impl TryFrom<tonic::Response<crate::proto::Response>> for ResultSet {
    type Error = ConversionError;

    /// Converts a gRPC response received from the Stargate coordinator
    /// into a `ResultSet`.
    ///
    /// Will return a `ConversionError` if the response does not contain a `ResultSet` message.
    fn try_from(response: tonic::Response<Response>) -> Result<Self, Self::Error> {
        match &response.get_ref().result {
            Some(crate::proto::response::Result::ResultSet(payload)) => {
                use prost::Message;
                let data: &prost_types::Any = payload.data.as_ref().unwrap();
                ResultSet::decode(data.value.as_slice())
                    .map_err(|e| ConversionError::decode_error::<_, Self>(response, e))
            }
            other => Err(ConversionError::incompatible::<_, Self>(other)),
        }
    }
}

impl Row {
    /// Takes a value of a single column at a given index and converts it to the desired type.
    ///
    /// This function does not copy the value so it should be quite cheap.
    /// The value gets moved out of the row and the original value remains empty.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{Row, Value};
    ///
    /// let mut row = Row {
    ///     values: vec![Value::int(1), Value::string("foo")]
    /// };
    ///
    /// let id: i64 = row.try_take(0).unwrap();
    /// let login: String = row.try_take(1).unwrap();
    ///
    /// assert_eq!(id, 1);
    /// assert_eq!(login, "foo".to_string());
    ///
    /// // Beware that the original row got erased:
    /// assert_eq!(row.values[0].inner, None);
    /// assert_eq!(row.values[1].inner, None);
    /// ```
    ///
    pub fn try_take<T: TryFromValue>(&mut self, at: usize) -> Result<T, ConversionError> {
        let len = self.values.len();
        if at >= len {
            Err(ConversionError::wrong_number_of_items::<_, T>(
                self, len, at,
            ))
        } else {
            self.values[at].take().try_into()
        }
    }

    /// Returns a copy of a value at a given index converted to the desired type.
    ///
    /// Unlike [`Row::try_take`], this function does not modify the original row, at the
    /// expense of making a deep copy of the value.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{Row, Value};
    ///
    /// let row = Row {
    ///     values: vec![Value::int(1), Value::string("foo")]
    /// };
    ///
    /// let id: i64 = row.try_get(0).unwrap();
    /// let login: String = row.try_get(1).unwrap();
    ///
    /// assert_eq!(id, 1);
    /// assert_eq!(login, "foo".to_string());
    pub fn try_get<T: TryFromValue>(&self, at: usize) -> Result<T, ConversionError> {
        let len = self.values.len();
        if at >= len {
            Err(ConversionError::wrong_number_of_items::<_, T>(
                self, len, at,
            ))
        } else {
            self.values[at].clone().try_into()
        }
    }
}

/// Error returned when a `ResultSetMapper` could not be constructed.
#[derive(Debug)]
pub enum MapperError {
    ColumnNotFound(&'static str),
}

impl Display for MapperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MapperError::ColumnNotFound(name) => {
                write!(f, "Column {} not found in the ResultSet", name)
            }
        }
    }
}

impl Error for MapperError {}

/// Matches the fields of the `Self` type to the column positions provided in the map.
pub trait ColumnPositions {
    fn field_to_column_pos(
        column_positions: HashMap<String, usize>,
    ) -> Result<Vec<usize>, MapperError>;
}

/// Converts rows to values of user type
pub trait TryFromRow
where
    Self: Sized,
{
    /// Attempts to convert a `Row` into `Self`.
    ///
    /// # Parameters
    /// - `row`: the row to convert
    /// - `column_positions`: the positions of values in the row for each field of `Self` type
    ///
    /// # Errors
    /// Failures to convert a row value must be signalled as `ConversionError`.
    ///
    /// # Panics
    /// This function is allowed to panic if the row is not large enough to contain the item
    /// at maximum index pointed to by `column_positions`.
    fn try_unpack(row: Row, column_positions: &[usize]) -> Result<Self, ConversionError>;
}

/// `ResultSetMapper` coverts a `Row` into `T`.
///
/// Call [`ResultSet::mapper`] to obtain one.
///
/// `ResultSetMapper` contains metadata required to map fields of `T` to
/// row columns, shared by all rows.
///
/// Please note that we cannot use standard `Into` and `From` traits to convert structs
/// from/to `Row`, because this transformation requires
/// additional context: the specification of columns, which is not included in a `Row`.
pub struct ResultSetMapper<T> {
    // The row columns might be ordered differently than the fields in the struct T.
    // This vector entries correspond to the fields in the struct.
    // Values in the vector denote positions of the columns.
    // E.g. `vec![1, 0]` would map field 0 to column 1 and field 1 to column 0.
    field_to_column_pos: Vec<usize>,
    // The minimum number of items in a row that we need to be able to unpack it
    required_row_len: usize,
    phantom_data: PhantomData<T>,
}

impl<T: TryFromRow> ResultSetMapper<T> {
    /// Attempts to convert the `row` into `T`.
    ///
    /// The row is allowed to contain additional values
    /// not needed by `T` - they will be ignored and lost, because you can't access row afterwards.
    ///
    /// # Errors
    /// Returns `ConversionError` if there are not enough values in the `row` or if any of the
    /// values fail to convert.
    /// If a value of the row fails to convert, the original error is returned.
    ///
    /// # Panics
    /// This function must not panic.
    pub fn try_unpack(&self, row: Row) -> Result<T, ConversionError> {
        let actual_len = row.values.len();
        if actual_len < self.required_row_len {
            return Err(ConversionError::wrong_number_of_items::<_, T>(
                row,
                actual_len,
                self.required_row_len,
            ));
        }
        <T as TryFromRow>::try_unpack(row, &self.field_to_column_pos)
    }
}

impl ResultSet {
    /// Creates a mapper that can convert `Row`s to values of type `T`.
    ///
    /// The mapper can be obtained for types that implement the `TryFromRow` and
    /// `ColumnPositions` traits. You can derive these traits for your structs with the
    /// `stargate_grpc_derive::TryFromRow` macro.
    ///
    /// # Errors
    /// The mapper creation will fail if the `ResultSet` metadata does not
    /// contain all columns required to construct values of type `T`.
    ///
    /// # Limitations
    /// Column types are not checked. If a column type does not match the field type in `T`
    /// the error will be signalled by [`ResultSetMapper::try_unpack`].
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::*;
    /// use stargate_grpc::proto::*;
    ///
    /// fn column(name: &str) -> ColumnSpec {
    ///     ColumnSpec {
    ///         r#type: None,
    ///         name: name.to_string(),
    ///     }
    /// }
    /// let result_set = ResultSet {
    ///     columns: vec![column("id"), column("login")],
    ///     rows: vec![Row {
    ///         values: vec![Value::int(1), Value::string("user_1")],
    ///     }],
    ///     paging_state: None,
    /// };
    ///
    /// #[derive(TryFromRow)]
    /// struct User {
    ///     id: i64,
    ///     login: String,
    /// }
    ///
    /// let mapper = result_set.mapper().unwrap();
    /// for row in result_set.rows {
    ///     let user: User = mapper.try_unpack(row).unwrap();
    ///     assert_eq!(user.id, 1);
    ///     assert_eq!(user.login, "user_1");
    /// }
    /// ```
    pub fn mapper<T>(&self) -> Result<ResultSetMapper<T>, MapperError>
    where
        T: ColumnPositions + TryFromRow,
    {
        let positions = <T as ColumnPositions>::field_to_column_pos(self.column_positions())?;
        Ok(ResultSetMapper {
            required_row_len: positions.iter().max().map(|m| *m + 1).unwrap_or(0),
            field_to_column_pos: positions,
            phantom_data: Default::default(),
        })
    }

    /// Returns a mapping from column names to column positions.
    /// The first column starts at position 0.
    fn column_positions(&self) -> HashMap<String, usize> {
        let mut result = HashMap::new();
        for (i, column) in self.columns.iter().enumerate() {
            result.insert(column.name.clone(), i);
        }
        result
    }
}
