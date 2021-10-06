//! Errors returned from failed attempts to convert data.

use prost::DecodeError;
use std::fmt::{Debug, Display, Formatter};

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

    /// When the source value is out of range of the target type.
    OutOfRange,

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

    pub fn incompatible<S: Debug, T>(source: S) -> ConversionError {
        Self::new::<S, T>(ConversionErrorKind::Incompatible, source)
    }

    pub fn out_of_range<S: Debug, T>(source: S) -> ConversionError {
        Self::new::<S, T>(ConversionErrorKind::OutOfRange, source)
    }

    pub fn wrong_number_of_items<S: Debug, T>(
        source: S,
        actual: usize,
        expected: usize,
    ) -> ConversionError {
        Self::new::<S, T>(
            ConversionErrorKind::WrongNumberOfItems { actual, expected },
            source,
        )
    }

    pub fn decode_error<S: Debug, T>(source: S, error: DecodeError) -> ConversionError {
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
