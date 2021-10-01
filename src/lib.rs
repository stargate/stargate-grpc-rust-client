use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use prost::DecodeError;
use regex::Regex;
use tonic::{Request, Status};
use tonic::codegen::{InterceptedService, StdError};
use tonic::metadata::AsciiMetadataValue;
use tonic::service::Interceptor;

pub use from_value::*;
pub use into_value::*;

mod from_value;
mod into_value;

tonic::include_proto!("stargate");

/// Error returned on an attempt to create an [`AuthToken`] from an invalid string.
#[derive(Clone, Debug)]
pub struct InvalidAuthToken(String);

impl Display for InvalidAuthToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid authentication token value format. Token must be an UUID."
        )
    }
}

impl Error for InvalidAuthToken {}

/// Stores a token for authenticating to Stargate.
///
/// You can obtain the token by sending a POST request with a username and password
/// to "/v1/auth" on port 8081 of Stargate.
///
/// # Example
/// <pre>
/// curl -L -X POST 'http://127.0.0.2:8081/v1/auth' \
///      -H 'Content-Type: application/json' \
///      --data-raw '{
///          "username": "cassandra",
///          "password": "cassandra"
///      }'
///
/// {"authToken":"25b538f6-3092-4fd1-8dd4-e73408f2bd60"}
/// </pre>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AuthToken(AsciiMetadataValue);

impl FromStr for AuthToken {
    type Err = InvalidAuthToken;

    /// Creates a new authentication token from a String.
    /// This will fail if the string is not a valid UUID.
    fn from_str(s: &str) -> Result<AuthToken, InvalidAuthToken> {
        let pattern =
            r"^[[:xdigit:]]{8}-[[:xdigit:]]{4}-[[:xdigit:]]{4}-[[:xdigit:]]{4}-[[:xdigit:]]{12}$";
        let pattern = Regex::new(pattern).unwrap();
        if pattern.is_match(s) {
            Ok(AuthToken(AsciiMetadataValue::from_str(s).unwrap()))
        } else {
            Err(InvalidAuthToken(s.to_string()))
        }
    }
}

/// Allows to use `AuthToken` as a Tonic request interceptor that
/// attaches its token value to request header "x-cassandra-token".
impl Interceptor for AuthToken {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let mut request = request;
        request
            .metadata_mut()
            .insert("x-cassandra-token", self.0.clone());
        Ok(request)
    }
}

/// Type alias for the most commonly used `StargateClient` type
/// with support for authentication.
pub type StargateClient =
    stargate_client::StargateClient<InterceptedService<tonic::transport::Channel, AuthToken>>;

impl StargateClient {
    /// Obtains a new `StargateClient` that attaches the authentication `token` to each request.
    pub async fn connect_with_auth<D>(
        dst: D,
        token: AuthToken,
    ) -> Result<Self, tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
        Ok(stargate_client::StargateClient::with_interceptor(
            conn, token,
        ))
    }
}

/// Error thrown when some data received from the wire could not be properly
/// converted to a desired Rust type.
#[derive(Clone, Debug)]
pub struct ConversionError {
    /// Describes the reason why the conversion failed
    pub kind: ConversionErrorKind,
    /// Debug string of the source value that failed to be converted
    pub source: String,
    /// Name of the target Rust type that the value failed to convert to
    pub target_type_name: String,
}

#[derive(Clone, Debug)]
pub enum ConversionErrorKind {
    /// When the converter didn't know how to convert one type to another
    /// because the conversion hasn't been defined
    Incompatible,

    /// When the number of elements in a vector or a tuple
    /// does not match the expected number of elements.
    WrongNumberOfItems { actual: usize, expected: usize },

    /// When the converter attempted to decode a binary blob,
    /// but the conversion failed due to invalid data
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

    fn wrong_number_of_items<S: Debug, T>(source: S, actual: usize, expected: usize) -> ConversionError {
        Self::new::<S, T>(ConversionErrorKind::WrongNumberOfItems { actual, expected }, source)
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
            self.source,
            self.target_type_name
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
                ResultSet::decode(data.value.as_slice()).map_err(|e| {
                    ConversionError::decode_error::<_, Self>( response, e)
                })
            }
            other => Err(ConversionError::incompatible::<_, Self>(other))
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
            Err(ConversionError::wrong_number_of_items::<_, Self>(self, len, 1))
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
            Err(ConversionError::wrong_number_of_items::<_, T>(self, len, index))
        } else {
            self.values[index].clone().try_into()
        }
    }
}
