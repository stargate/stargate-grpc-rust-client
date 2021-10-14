//! Structs for describing the types of gRPC data values
//!
//! These structs do not hold any data, they exist purely for
//! describing types. They are needed for constructing type parameters passed to
//! [`Value::of_type`](crate::Value::of_type) or [`Value::list_of`](crate::Value::list_of)
//! functions.
//!
//! # Example
//! ```
//! use stargate_grpc::types;
//!
//! let int_type = types::Int;
//! let list_of_ints = types::List(types::Int);
//! let list_of_tuples = types::List((types::Int, types::String));
//! let map_from_uuid_to_user_type = types::Map(types::Uuid, types::Udt);
//! ```

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

pub struct Timestamp;
impl ConcreteType for Timestamp {}

pub struct Udt;
impl ConcreteType for Udt {}

pub struct Uuid;
impl ConcreteType for Uuid {}

pub struct Varint;
impl ConcreteType for Varint {}

pub struct List<T>(pub T);
impl<T> ConcreteType for List<T> {}

pub struct Set<T>(pub T);
impl<T> ConcreteType for Set<T> {}

pub struct Map<K, V>(pub K, pub V);
impl<K, V> ConcreteType for Map<K, V> {}

/// Used in target type specification passed to [`Value::of_type`](crate::Value::of_type)
/// to mark that the conversion should generate a `Value` of the default type.
/// It is handy if we already have a `Value` in the structure to be converted, and we
/// just want it to be passed-through.
pub struct Any;
