use std::convert::TryFrom;

pub use from_value::*;
pub use into_value::*;
use prost::DecodeError;
use std::fmt::{Debug, Display, Formatter};

mod from_value;
mod into_value;

tonic::include_proto!("stargate");

/// Error thrown when some data received from the wire could not be properly
/// converted to a desired Rust type.
#[derive(Clone, Debug)]
pub struct ConversionError {
    kind: ConversionErrorKind,
    value: String,
    rust_type_name: &'static str,
}

#[derive(Clone, Debug)]
pub enum ConversionErrorKind {
    /// When the converter didn't know how to convert one type to another because the conversion
    /// hasn't been defined
    NoRecipe,
    /// When the converter attempted to decode a binary blob, but conversion failed due to
    /// invalid data
    GrpcDecodeError(DecodeError),
}

impl ConversionError {
    fn no_recipe<T, V: Debug>(cql_value: V) -> ConversionError {
        ConversionError {
            kind: ConversionErrorKind::NoRecipe,
            value: format!("{:?}", cql_value),
            rust_type_name: std::any::type_name::<T>(),
        }
    }

    fn decode_error<T, V: Debug>(cql_value: V, cause: DecodeError) -> ConversionError {
        ConversionError {
            kind: ConversionErrorKind::GrpcDecodeError(cause),
            value: format!("{:?}", cql_value),
            rust_type_name: std::any::type_name::<T>(),
        }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot convert value {} to {}",
            self.value, self.rust_type_name
        )
    }
}

/// Defines structs for describing the gRPC data types that user data passed inside `Value`
/// should be converted into. These structs do not hold any data, they exist purely for
/// describing types. They are needed for constructing type parameters passed to
/// [`Value::of_type`](crate::Value::of_type) or [`Value::list_of`](crate::Value::list_of)
/// functions.
///
/// # Example
/// ```
/// use stargate_grpc::types;
///
/// let int_type = types::Int;
/// let list_of_ints = types::List(types::Int);
/// let list_of_tuples = types::List((types::Int, types::String));
/// let map_from_uuid_to_user_type = types::Map(types::Uuid, types::Udt);
/// ```
pub mod types {

    /// Must be implemented by all types except Any.
    pub trait ConcreteType {}

    pub struct Boolean;
    impl ConcreteType for Boolean {}
    pub struct Bytes;
    impl ConcreteType for Bytes {}
    pub struct Date;
    impl ConcreteType for Date {}
    pub struct Decimal;
    impl ConcreteType for Decimal {}
    pub struct Double;
    impl ConcreteType for Double {}
    pub struct Float;
    impl ConcreteType for Float {}
    pub struct Inet;
    impl ConcreteType for Inet {}
    pub struct Int;
    impl ConcreteType for Int {}
    pub struct String;
    impl ConcreteType for String {}
    pub struct Time;
    impl ConcreteType for Time {}
    pub struct Udt;
    impl ConcreteType for Udt {}
    pub struct Uuid;
    impl ConcreteType for Uuid {}
    pub struct Varint;
    impl ConcreteType for Varint {}

    pub struct List<T>(pub T);
    impl<T> ConcreteType for List<T> {}

    pub struct Map<K, V>(pub K, pub V);
    impl<K, V> ConcreteType for Map<K, V> {}

    /// Used in target type specification passed to [`Value::of_type`](crate::Value::of_type())
    /// to mark that the conversion should generate a `Value` of the default type.
    /// It is handy if we already have a `Value` in the structure to be converted, and we
    /// just want it to be passed-through.
    pub struct Any;
}

/// A handy conversion that let us get directly to the `ResultSet` returned by a query.
impl TryFrom<tonic::Response<crate::Response>> for ResultSet {
    type Error = ConversionError;

    fn try_from(response: tonic::Response<Response>) -> Result<Self, Self::Error> {
        match &response.get_ref().result {
            Some(response::Result::ResultSet(payload)) => {
                use prost::Message;
                let data: &prost_types::Any = payload.data.as_ref().unwrap();
                ResultSet::decode(data.value.as_slice())
                    .map_err(|e| ConversionError::decode_error::<ResultSet, _>(response, e))
            }
            other => Err(ConversionError::no_recipe::<ResultSet, _>(other)),
        }
    }
}
