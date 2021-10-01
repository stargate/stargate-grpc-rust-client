use crate::{ConversionError, Response, ResultSet, Row, TryFromValue};
use std::convert::TryFrom;

/// A handy conversion that let us get directly to the `ResultSet` returned by a query.
impl TryFrom<tonic::Response<crate::Response>> for ResultSet {
    type Error = ConversionError;

    fn try_from(response: tonic::Response<Response>) -> Result<Self, Self::Error> {
        match &response.get_ref().result {
            Some(crate::response::Result::ResultSet(payload)) => {
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
    pub fn into_single<T: TryFromValue>(self) -> Result<T, ConversionError> {
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
