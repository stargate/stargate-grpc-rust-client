//! Automatic data type conversions and utilities useful for processing query results.

use crate::error::ConversionError;
use crate::from_value::TryFromValue;
use crate::proto::{Response, ResultSet, Row};

use std::convert::TryFrom;

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
    /// Converts the row containing a single value into the desired type.
    ///
    /// Returns `ConversionError` if the row doesn't contain exactly one value or if a value
    /// could not be converted to `T`.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{Row, Value};
    ///
    /// let row1 = Row {
    ///     values: vec![Value::list(vec![1, 2, 3])]
    /// };
    /// let row2 = row1.clone();
    ///
    /// let vector: Vec<i64> = row1.try_into_one().unwrap();
    /// assert_eq!(vector, vec![1, 2, 3]);
    ///
    /// let tuple: (i64, i64, i64) = row2.try_into_one().unwrap();
    /// assert_eq!(tuple, (1, 2, 3));
    /// ```
    pub fn try_into_one<T: TryFromValue>(self) -> Result<T, ConversionError> {
        let len = self.values.len();
        if len != 1 {
            Err(ConversionError::wrong_number_of_items::<_, Self>(
                self, len, 1,
            ))
        } else {
            self.values.into_iter().next().unwrap().try_into()
        }
    }

    /// Returns a value of a single column converted to a desired type.
    ///
    /// This function does not move the value out of the row. It makes a copy before
    /// the conversion instead.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{Row, Value};
    ///
    /// let row = Row {
    ///     values: vec![Value::int(1), Value::string("foo")]
    /// };
    ///
    /// let id: i64 = row.get(0).unwrap();
    /// let login: String = row.get(1).unwrap();
    ///
    /// assert_eq!(id, 1);
    /// assert_eq!(login, "foo".to_string());
    /// ```
    ///
    pub fn get<T: TryFromValue>(&self, index: usize) -> Result<T, ConversionError> {
        let len = self.values.len();
        if index >= len {
            Err(ConversionError::wrong_number_of_items::<_, T>(
                self, len, index,
            ))
        } else {
            self.values[index].clone().try_into()
        }
    }
}
