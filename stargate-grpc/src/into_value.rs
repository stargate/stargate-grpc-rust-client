//! # Automatic conversions from standard Rust types to `Value`.
//!
//! Values can be obtained by calling dedicated factory methods:
//! ```rust
//! # use stargate_grpc::Value;
//! #
//! let int_value = Value::int(5);
//! let double_value = Value::double(3.14);
//! let string_value = Value::string("foo");
//! let inet_value = Value::inet(&[127, 0, 0, 1]);
//! let bytes_value = Value::bytes(vec![0xff, 0xfe, 0x00]);
//! let list_value = Value::list(vec![1.41, 3.14]);
//! let heterogeneous = Value::list(vec![Value::int(1), Value::double(3.14)]);
//! ```
//! Values can be also generically created from commonly used Rust types using
//! standard Rust [`Into`](std::convert::Into) or [`From`](std::convert::From) traits:
//! ```rust
//! # use stargate_grpc::Value;
//! #
//! let int_value = Value::from(5);
//! let int_value: Value = 5.into();
//!
//! let string_value = Value::from("stargate");
//! let string_value: Value = "stargate".into();
//!
//! let list1 = Value::from(vec![1, 2]);
//! let list1: Value = vec![1, 2].into();
//!
//! let list2 = Value::from((1, 3.14));
//! let list2: Value = (1, 3.14).into();
//! ```
//!
//! It is also possible to specify the desired target CQL type by using [`Value::of_type`]
//! to disambiguate when more than one target types are possible:
//! ```rust
//! # use stargate_grpc::{types, Value};
//! #
//! let bytes = Value::of_type(types::List(types::Bytes), vec![vec![0, 1], vec![2, 3]]);
//! let ints = Value::of_type(types::List(types::Varint), vec![vec![0, 1], vec![2, 3]]);
//! assert_ne!(bytes, ints);
//! ```
//! Specifying the desired target type is more type safe and may guard you from
//! sending the data of a wrong type:
//! ```ignore
//! let list_of_strings = Value::of_type(types::List(types::String), vec![10]); // compile time error
//! ```
//! ## Standard conversions
//! | Rust type                     | gRPC type
//! |-------------------------------|------------------------------------
//! | `i8`                          | [`types::Int`]
//! | `i16`                         | [`types::Int`]
//! | `i32`                         | [`types::Int`]
//! | `i64`                         | [`types::Int`]
//! | `u16`                         | [`types::Int`]
//! | `u32`                         | [`types::Int`], [`types::Date`]
//! | `u64`                         | [`types::Time`]
//! | `f32`                         | [`types::Float`]
//! | `f64`                         | [`types::Double`]
//! | `bool`                        | [`types::Boolean`]
//! | `String`                      | [`types::String`]
//! | `&str`                        | [`types::String`]
//! | `std::time::SystemTime`       | [`types::Int`]
//! | `Vec<u8>`                     | [`types::Bytes`, `types::Varint`]
//! | `Vec<T>`                      | [`types::List`]
//! | `Vec<(K, V)>`                 | [`types::Map`]
//! | `Vec<KeyValue>`               | [`types::Map`]
//! | `HashMap<K, V>`               | [`types::Map`]
//! | `BTreeMap<K, V>`              | [`types::Map`]
//! | `(T1, T2, ...)`               | [`types::List`]
//! | &[u8; 4]                      | [`types::Inet`]
//! | &[u8; 16]                     | [`types::Inet`]
//! | &[u8; 16]                     | [`types::Uuid`]
//! | [`proto::Decimal`]            | [`types::Decimal`]
//! | [`proto::Inet`]               | [`types::Inet`]
//! | [`proto::UdtValue`]           | [`types::Udt`]
//! | [`proto::Uuid`]               | [`types::Uuid`]
//! | [`proto::Varint`]             | [`types::Varint`]
//!
//!
//! ## Optional conversions
//!
//! The following conversions are provided by features `chrono` and `uuid`:
//!
//! | Rust type                   | gRPC type
//! |-----------------------------|------------------------------------
//! | `chrono::Date<T>`           | [`types::Date`]
//! | `chrono::DateTime<T>`       | [`types::Int`]
//! | `uuid::Uuid`                | [`types::Uuid`]
//!
//!
//! ## Collections
//!
//! Elements inside of collections are converted to default `Value` types automatically.
//! This applies to nested collections as well.
//!
//! You may have noticed that this crate defines two types that are not present in the gRPC
//! protocol: `types::List` and `types::Map`. Collections of both of those types are internally
//! mapped to an [Inner::Collection](proto::value::Inner::Collection) variant.
//! However, the distinction between a map and a list is needed in order to allow you to
//! specify the type of map's keys separately from the type of the values.
//!
//! ```rust
//! use std::collections::HashMap;
//! use stargate_grpc::{types, Value};
//!
//! let mut dates = HashMap::new();
//! dates.insert("start", 18740);   // days since Unix epoch
//! dates.insert("end", 18747);
//!
//! let date_map = Value::of_type(types::Map(types::String, types::Date), dates);
//! ```
//!
//! By specifying a target type as `types::Map` you're also able to convert a vector of pairs
//! into a collection representing a map, although the default target type for converting a
//! `Vec<T>` is a list:
//!
//! ```rust
//! use stargate_grpc::{types, Value};
//!
//! let collection1 = vec![("key1", 1), ("key2", 2)];
//! let collection2 = collection1.clone();
//!
//! // Maps to map<string, bigint> on the C* side:
//! let value_as_map = Value::of_type(types::Map(types::String, types::Int), collection1);
//! // Maps to list<tuple<string, bigint>> on the C* side:
//! let value_as_list = Value::of_type(types::List((types::String, types::Int)), collection2);
//!
//! assert_ne!(value_as_map, value_as_list)
//! ```
//!
//! ## Converting from `chrono::Date` and `chrono::DateTime`
//!
//! In order to be able to convert `chrono` dates and timestamps into `Value`,
//! add `chrono` crate to dependencies of your project and enable `chrono` feature on this crate.
//! All `chrono` timezones are supported.
//!
//! ```rust
//! # #[cfg(feature = "chrono")] {
//! # use stargate_grpc::Value;
//! let timestamp = Value::from(chrono::Utc::now());
//! let today = Value::from(chrono::Utc::now().date());
//! # }
//! ```
//!
//! ## Converting from `uuid::Uuid`
//!
//! In order to be able to convert `uuid` UUIDs into `Value`
//! add `uuid` crate to dependencies and enable `uuid` feature on this crate.
//! All UUID types are supported.
//!
//! ```rust
//! # #[cfg(feature = "uuid")] {
//! # use stargate_grpc::Value;
//! let uuid = Value::from(uuid::Uuid::new_v4());
//! # }
//!```
//!
//! ## Custom conversions
//! You can make any type convertible to `Value` by implementing the [`IntoValue`] trait.
//! Use one of `Value::raw_` methods to construct the actual value.
//!
//! Provide a [`DefaultGrpcType`] to make the conversion to desired gRPC type be chosen
//! automatically even when the target value type is not known.
//! If the default type is specified, you'll also get implementations of appropriate
//! [`std::convert::From`] and [`std::convert::Into`] traits for free.
//!
//! For example, let's define such conversion from a custom `Login` struct` that wraps a `String`:
//!```
//! use stargate_grpc::into_value::{DefaultGrpcType, IntoValue};
//! use stargate_grpc::{types, Value};
//!
//! struct Login(String);
//!
//! impl IntoValue<types::String> for Login {
//!    fn into_value(self) -> Value {
//!        Value::raw_string(self.0)
//!    }
//! }
//!
//! impl DefaultGrpcType for Login {
//!    type C = types::String;
//! }
//!
//! let login = Login("login".to_string());
//! assert_eq!(Value::string(login), Value::string("login"));
//!
//! let login = Login("login".to_string());
//! assert_eq!(Value::from(login), Value::string("login"));
//! ```
//!

use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::hash::Hash;
use std::time::SystemTime;

use itertools::Itertools;

use crate::types::ConcreteType;
use crate::*;

/// Selects the default Cassandra gRPC value type associated with a Rust type.
/// The default type is used when a Rust value `x` is converted to `Value` by calling
/// `x.into()` or `Value::from(x)`.
///
/// In order to convert a Rust value to a non-default Cassandra type, or to convert
/// a Rust type that doesn't have a default conversion defined, use [`Value::of_type`].
///
/// # Example
/// ```
/// use stargate_grpc::Value;
///
/// assert_eq!(Value::from(true), Value::boolean(true));
/// assert_eq!(Value::from(1.0), Value::double(1.0));
/// assert_eq!(Value::from(vec![1, 2]), Value::list(vec![Value::int(1), Value::int(2)]));
///
/// let x: Value = 100.into();
/// assert_eq!(x, Value::int(100));
///
/// let some: Value = Some(100).into();
/// assert_eq!(some, Value::int(100));
/// let none: Value = (None as Option<i32>).into();
/// assert_eq!(none, Value::null());
///
/// ```
pub trait DefaultGrpcType {
    /// gRPC type, must be set to one of the types defined in the [`types`](crate::types) module.
    type C;
}

impl DefaultGrpcType for bool {
    type C = types::Boolean;
}

impl DefaultGrpcType for i8 {
    type C = types::Int;
}

impl DefaultGrpcType for i16 {
    type C = types::Int;
}

impl DefaultGrpcType for i32 {
    type C = types::Int;
}

impl DefaultGrpcType for i64 {
    type C = types::Int;
}

impl DefaultGrpcType for u16 {
    type C = types::Int;
}

impl DefaultGrpcType for u32 {
    type C = types::Int;
}

impl DefaultGrpcType for f32 {
    type C = types::Float;
}

impl DefaultGrpcType for f64 {
    type C = types::Double;
}

impl DefaultGrpcType for String {
    type C = types::String;
}

impl DefaultGrpcType for &str {
    type C = types::String;
}

impl DefaultGrpcType for Vec<u8> {
    type C = types::Bytes;
}

impl DefaultGrpcType for proto::Decimal {
    type C = types::Decimal;
}

impl DefaultGrpcType for proto::Inet {
    type C = types::Inet;
}

impl DefaultGrpcType for proto::UdtValue {
    type C = types::Udt;
}

impl DefaultGrpcType for proto::Uuid {
    type C = types::Uuid;
}

#[cfg(feature = "uuid")]
impl DefaultGrpcType for uuid::Uuid {
    type C = types::Uuid;
}

impl DefaultGrpcType for proto::Varint {
    type C = types::Varint;
}

impl DefaultGrpcType for SystemTime {
    type C = types::Int;
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> DefaultGrpcType for chrono::DateTime<Tz> {
    type C = types::Int;
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> DefaultGrpcType for chrono::Date<Tz> {
    type C = types::Date;
}

impl<T> DefaultGrpcType for Option<T>
where
    T: DefaultGrpcType,
{
    type C = <T as DefaultGrpcType>::C;
}

impl<T> DefaultGrpcType for Vec<T>
where
    T: DefaultGrpcType,
{
    type C = types::List<<T as DefaultGrpcType>::C>;
}

impl<K, V> DefaultGrpcType for Vec<KeyValue<K, V>>
where
    K: DefaultGrpcType,
    V: DefaultGrpcType,
{
    type C = types::Map<<K as DefaultGrpcType>::C, <V as DefaultGrpcType>::C>;
}

impl<K, V> DefaultGrpcType for HashMap<K, V>
where
    K: DefaultGrpcType,
    V: DefaultGrpcType,
{
    type C = types::Map<<K as DefaultGrpcType>::C, <V as DefaultGrpcType>::C>;
}

impl<K, V> DefaultGrpcType for BTreeMap<K, V>
where
    K: DefaultGrpcType,
    V: DefaultGrpcType,
{
    type C = types::Map<<K as DefaultGrpcType>::C, <V as DefaultGrpcType>::C>;
}

/// Converts a value of Rust type into a Value of given Cassandra type.
///
/// Thanks to additional type parameter `C`, it is possible to define multiple conversions
/// from a single Rust type to many Cassandra types. E.g. a `Vec<u8>` can be converted
/// to `Bytes`, `Inet` or `Varint`.
///
/// # Type arguments
/// - `C` - Cassandra type represented by a struct defined in the `types` module;
pub trait IntoValue<C> {
    fn into_value(self) -> Value;
}

impl Value {
    /// Constructs a CQL boolean value without applying additional conversions.
    /// CQL type: `boolean`.
    pub fn raw_boolean(value: bool) -> Value {
        Value {
            inner: Some(proto::value::Inner::Boolean(value)),
        }
    }

    /// Constructs an integer value without applying additional conversions.
    /// CQL types: `tinyint`, `smallint`, `int`, `bigint`, `counter`, `timestamp`.
    pub fn raw_int(value: i64) -> Value {
        Value {
            inner: Some(proto::value::Inner::Int(value)),
        }
    }

    /// Constructs a float value without applying conversions.
    /// CQL types: `float`.
    pub fn raw_float(value: f32) -> Value {
        Value {
            inner: Some(proto::value::Inner::Float(value)),
        }
    }

    /// Constructs a double value without applying additional conversions.
    /// CQL types: `double`.
    pub fn raw_double(value: f64) -> Value {
        Value {
            inner: Some(proto::value::Inner::Double(value)),
        }
    }

    /// Constructs a date value from the number of days since Unix epoch.
    /// Doesn't apply additional conversions.
    /// CQL types: `date`.
    pub fn raw_date(value: u32) -> Value {
        Value {
            inner: Some(proto::value::Inner::Date(value)),
        }
    }

    /// Constructs a date value from the number of nanoseconds since midnight.
    /// Doesn't apply additional conversions.
    /// CQL types: `time`.
    pub fn raw_time(value: u64) -> Value {
        Value {
            inner: Some(proto::value::Inner::Time(value)),
        }
    }

    /// Constructs a UUID value from raw bytes without applying additional conversions.
    /// CQL types: `uuid`, `timeuuid`.
    pub fn raw_uuid(value: &[u8; 16]) -> Value {
        Value {
            inner: Some(proto::value::Inner::Uuid(proto::Uuid {
                value: value.to_vec(),
            })),
        }
    }

    /// Constructs an internet address value from raw bytes, with applying additional conversions.
    /// CQL types: `inet`.
    pub fn raw_inet(value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Inet(proto::Inet { value })),
        }
    }

    /// Constructs a binary blob value by wrapping bytes, without applying additional conversions.
    /// CQL types: `blob`, `custom`.
    pub fn raw_bytes(value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Bytes(value)),
        }
    }

    /// Constructs a variable length interger from raw byte representation.
    /// CQL types: `varint`.
    pub fn raw_varint(value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Varint(proto::Varint { value })),
        }
    }

    /// Constructs a decimal value from raw mantissa and scale.
    /// CQL types: `decimal`.
    pub fn raw_decimal(scale: u32, value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Decimal(proto::Decimal {
                scale,
                value,
            })),
        }
    }

    /// Constructs a string value from any value that can be converted to a String.
    /// CQL types: `string`, `varchar`
    pub fn raw_string<S: ToString>(value: S) -> Value {
        Value {
            inner: Some(proto::value::Inner::String(value.to_string())),
        }
    }

    /// Constructs a collection of values.
    /// CQL types: `list`, `set`, `map`, `tuple`.
    pub fn raw_collection(elements: Vec<Value>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Collection(proto::Collection {
                elements,
            })),
        }
    }

    /// Constructs a user defined type value from field values.
    /// CQL types: user defined types
    pub fn raw_udt(fields: HashMap<String, Value>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Udt(proto::UdtValue { fields })),
        }
    }

    /// Converts a value of different type into a `Value`.
    ///
    /// Same as [`Value::of_type`] but doesn't require the type argument.
    /// Used internally to provide default type conversions.
    fn convert<R: IntoValue<C>, C>(value: R) -> Value {
        value.into_value()
    }

    /// Converts a Rust value to a `Value` of gRPC type specified by one of the types from
    /// the [`types`] module.
    ///
    /// Provides additional compile-time type safety by letting the compiler
    /// know the exact type of data that needs to be generated.
    /// If the desired type cannot be converted to, the code wouldn't compile,
    /// and you wouldn't waste time trying to run your queries with invalid data.
    ///
    /// Additionally, some Rust types allow more than one target gRPC type, e.g.
    /// `u32` can be converted to either an `Int` or a `Date`. This method allows to select
    /// a non-default target type in such case.
    /// Additionally, `u64` doesn't have the default conversion to `Time` defined in order
    /// to avoid accidental confusion with integers, so it also must be converted explicitly.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::Value;
    /// use stargate_grpc::types::*;
    ///
    /// let integers = Value::of_type(List(Int), vec![1, 2]);
    /// assert_eq!(integers, Value::list(vec![
    ///     Value::int(1),
    ///     Value::int(2)
    /// ]));
    ///
    /// let times_since_midnight = Value::of_type(List(Time), vec![1000, 2000]);
    /// assert_eq!(times_since_midnight, Value::list(vec![
    ///     Value::time(1000),
    ///     Value::time(2000)
    /// ]));
    ///
    /// // This wouldn't compile:
    /// // let strings = Value::of_type(List(String), vec![1, 2]);
    /// ```
    ///
    /// In cases where you don't want to describe the result type fully, or where you need
    /// to specify a heterogeneous collection of items of different types, you can use
    /// [`types::Any`](crate::types::Any) to let the compiler figure out the target type
    /// automatically based on the source type.
    ///
    /// # Example
    /// ```
    /// use std::collections::{BTreeMap};
    /// use stargate_grpc::Value;
    /// use stargate_grpc::types::{Map, Time, Any};
    ///
    /// // Create a map that holds values of different types
    /// let mut map = BTreeMap::new();
    /// map.insert(1, Value::int(1));
    /// map.insert(2, Value::string("foo"));
    ///
    /// // Specify the keys should be converted to time:
    /// let value = Value::of_type(Map(Time, Any), map);
    ///
    /// assert_eq!(value, Value::map(vec![
    ///     (Value::time(1), Value::int(1)),
    ///     (Value::time(2), Value::string("foo")),
    /// ]));
    /// ```
    pub fn of_type<R: IntoValue<C>, C>(_type_spec: C, value: R) -> Value {
        value.into_value()
    }

    /// Creates a CQL `null` value.
    pub fn null() -> Value {
        Value {
            inner: Some(proto::value::Inner::Null(proto::value::Null {})),
        }
    }

    /// Unset value. Unset query parameter values are ignored by the server.
    ///
    /// Use this value if you need to bind a parameter in an insert statement,
    /// but you don't want to change the target value stored in the database.
    /// To be used only for bind values in queries.
    pub fn unset() -> Value {
        Value {
            inner: Some(proto::value::Inner::Unset(proto::value::Unset {})),
        }
    }

    /// Constructs a CQL `boolean` value.
    pub fn boolean(value: impl IntoValue<types::Boolean>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `tinyint`, `smallint`, `int`, `bigint`, `counter` or `timestamp` value.
    pub fn int(value: impl IntoValue<types::Int>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `float` value.
    pub fn float(value: impl IntoValue<types::Float>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `double` value.
    pub fn double(value: impl IntoValue<types::Double>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `date` value.
    pub fn date(value: impl IntoValue<types::Date>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `time` value.
    pub fn time(value: impl IntoValue<types::Time>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `uuid` or `timeuuid` value.
    pub fn uuid(value: impl IntoValue<types::Uuid>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `inet` value.
    pub fn inet(value: impl IntoValue<types::Inet>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `blob` or `custom` value.
    pub fn bytes(value: impl IntoValue<types::Bytes>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `varint` value.
    pub fn varint(value: impl IntoValue<types::Varint>) -> Value {
        value.into_value()
    }

    /// Construcst a CQL `decimal` value.
    pub fn decimal(value: impl IntoValue<types::Decimal>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `ascii`, `varchar` or `text` value.
    pub fn string(value: impl IntoValue<types::String>) -> Value {
        value.into_value()
    }

    /// Constructs a CQL `list`, `set` or `tuple` value.
    ///
    /// Items are converted to `Value` using the default conversion associated
    /// with their actual type.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::Value;
    ///
    /// assert_eq!(
    ///     Value::list(vec![1, 2]),
    ///     Value::list(vec![Value::int(1), Value::int(2)])
    /// );
    /// ```
    /// See also [`Value::list_of`].
    pub fn list<I, T>(elements: I) -> Value
    where
        I: IntoIterator<Item = T>,
        T: Into<Value>,
    {
        Value::list_of(types::Any, elements)
    }

    /// Constructs a CQL `list`, `set` or `tuple` value.
    /// Allows to specify the target type of the elements.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{types, Value};
    ///
    /// assert_eq!(
    ///     Value::list_of(types::Time, vec![1, 2]),
    ///     Value::list(vec![Value::time(1), Value::time(2)])
    /// );
    pub fn list_of<E, I, T>(_element_type: E, elements: I) -> Value
    where
        I: IntoIterator<Item = T>,
        T: IntoValue<E>,
    {
        let elements = elements.into_iter().map(|e| e.into_value()).collect_vec();
        Value::raw_collection(elements)
    }

    /// Converts a collection of key-value pairs to a CQL `map` value.
    ///
    /// Keys and values of the map are converted to `Value` using the default conversions
    /// associated with their types. Keys may be converted to a different type than values.
    ///
    /// CQL type: `map`
    ///
    /// # Type Parameters
    /// - `I`: type of the collection
    /// - `K`: type of the keys in the input collection
    /// - `V`: type of the values in the input collection
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::Value;
    /// use std::collections::{BTreeMap};
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, "foo");
    /// map.insert(2, "bar");
    ///
    /// assert_eq!(
    ///     Value::map(map),
    ///     Value::map(vec![
    ///         (Value::int(1), Value::string("foo")),
    ///         (Value::int(2), Value::string("bar"))
    ///     ])
    /// );
    /// ```
    /// See also [`Value::map_of`].
    pub fn map<I, K, V>(key_value_pairs: I) -> Value
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Value>,
        V: Into<Value>,
    {
        Value::map_of(types::Any, types::Any, key_value_pairs)
    }

    /// Converts a collection of key-value pairs to a CQL `map` value.
    /// Allows to specify the target key and value types.
    ///
    /// Keys and values of the map are converted to CQL types specified by `CK` and `CV` types.
    /// The calling code will not compile if the elements of the map cannot be converted
    /// to given CQL types.
    ///
    /// # Type Parameters
    /// - `I`: type of the collection
    /// - `RK`: type of the keys in the input collection
    /// - `RV`: type of the values in the input collection
    /// - `CK`: desired gRPC type of the keys of the result map
    /// - `CV`: desired gRPC type of the values of the result map
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::{types, Value};
    /// use stargate_grpc::proto::Inet;
    /// use std::collections::{BTreeMap};
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, &[127, 0, 0, 1]);
    /// map.insert(2, &[127, 0, 0, 2]);
    ///
    /// assert_eq!(
    ///     Value::map_of(types::Int, types::Inet, map),
    ///     Value::map(vec![
    ///         (Value::int(1), Value::inet(&[127, 0, 0, 1])),
    ///         (Value::int(2), Value::inet(&[127, 0, 0, 2]))
    ///     ])
    /// );
    /// ```
    pub fn map_of<CK, CV, I, RK, RV>(_key_type: CK, _value_type: CV, elements: I) -> Value
    where
        I: IntoIterator<Item = (RK, RV)>,
        RK: IntoValue<CK>,
        RV: IntoValue<CV>,
    {
        let iter = elements.into_iter();
        let (size_hint_lower, size_hint_upper) = iter.size_hint();
        let mut collection = Vec::with_capacity(size_hint_upper.unwrap_or(size_hint_lower) * 2);
        for (k, v) in iter {
            collection.push(k.into_value());
            collection.push(v.into_value());
        }
        Value {
            inner: Some(proto::value::Inner::Collection(proto::Collection {
                elements: collection,
            })),
        }
    }

    /// Converts a collection of key-value pairs into a CQL value of a user defined type.
    ///
    /// Keys must be convertible to strings and values must be convertible to `Value`.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::Value;
    /// use std::collections::HashMap;
    ///
    /// let mut obj = HashMap::new();
    /// obj.insert("field1", 1);
    /// obj.insert("field2", 2);
    ///
    /// let udt_value = Value::udt(obj);
    /// ```
    pub fn udt<I, K, V>(fields: I) -> Value
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: Into<Value>,
    {
        let fields = fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.into()))
            .collect();
        Value::raw_udt(fields)
    }
}

impl<R> From<R> for Value
where
    R: DefaultGrpcType + IntoValue<<R as into_value::DefaultGrpcType>::C>,
{
    fn from(value: R) -> Self {
        Value::convert::<R, <R as DefaultGrpcType>::C>(value)
    }
}

/// When requested to convert a Rust type to a Value of type Any, just use the default conversion.
impl<R> IntoValue<types::Any> for R
where
    R: Into<Value>,
{
    fn into_value(self) -> Value {
        self.into()
    }
}

/// Generates a conversion from Rust concrete type to given Cassandra type.
///
/// # Parameters
/// - `R`: Rust type
/// - `C`: Cassandra gRPC data type (from `types`)
/// - `from`: original Rust value before the conversion
/// - `to`: expression yielding a `Value`
macro_rules! gen_conversion {
    ($R:ty => $C:ty; $from:ident => $to:expr) => {
        impl IntoValue<$C> for $R {
            fn into_value(self) -> Value {
                let $from = self;
                $to
            }
        }
    };
}

gen_conversion!(bool => types::Boolean; x => Value::raw_boolean(x));

gen_conversion!(i64 => types::Int; x => Value::raw_int(x));
gen_conversion!(i32 => types::Int; x => Value::raw_int(x as i64));
gen_conversion!(i16 => types::Int; x => Value::raw_int(x as i64));
gen_conversion!(i8 => types::Int; x => Value::raw_int(x as i64));

//there is no u64 to Int conversion because it doesn't fit fully in the target range
gen_conversion!(u32 => types::Int; x => Value::raw_int(x as i64));
gen_conversion!(u16 => types::Int; x => Value::raw_int(x as i64));
gen_conversion!(u8 => types::Int; x => Value::raw_int(x as i64));

gen_conversion!(u32 => types::Date; x => Value::raw_date(x));
gen_conversion!(u64 => types::Time; x => Value::raw_time(x));

gen_conversion!(f32 => types::Float; x => Value::raw_float(x));
gen_conversion!(f64 => types::Double; x => Value::raw_double(x));

gen_conversion!(String => types::String; x => Value::raw_string(x));
gen_conversion!(&str => types::String; x => Value::raw_string(x.to_string()));

gen_conversion!(Vec<u8> => types::Bytes; x => Value::raw_bytes(x));
gen_conversion!(Vec<u8> => types::Varint; x => Value::raw_varint(x));

gen_conversion!(&[u8; 4] => types::Inet; x => Value::raw_inet(x.to_vec()));
gen_conversion!(&[u8; 16] => types::Inet; x => Value::raw_inet(x.to_vec()));
gen_conversion!(&[u8; 16] => types::Uuid; x => Value::raw_uuid(x));

gen_conversion!(proto::Decimal => types::Decimal; x => Value::raw_decimal(x.scale, x.value));
gen_conversion!(proto::Inet => types::Inet; x => Value::raw_inet(x.value));
gen_conversion!(proto::UdtValue => types::Udt; x => Value::raw_udt(x.fields));
gen_conversion!(proto::Uuid => types::Uuid; x =>
    Value::raw_uuid(&x.value.try_into().expect("16 bytes")));
gen_conversion!(proto::Varint => types::Varint; x => Value::raw_varint(x.value));

gen_conversion!(SystemTime => types::Int; x =>
    Value::raw_int(x.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64));

#[cfg(feature = "uuid")]
gen_conversion!(uuid::Uuid => types::Uuid; x => Value::raw_uuid(x.as_bytes()));

/// Generates generic conversion from a Rust tuple to `Value`.
///
/// # Parameters:
/// - `index`: index of the tuple element, starts at 0
/// - `R`: type variable used to denote Rust type
/// - `C`: type variable used to denote Cassandra type
macro_rules! gen_tuple_conversion {
    ($($index:tt: $R:ident => $C:ident),+) => {

        impl <$($R),+, $($C),+> IntoValue<($($C),+)> for ($($R),+)
        where $($R: IntoValue<$C>),+
        {
            fn into_value(self) -> Value {
                Value::raw_collection(vec![$(self.$index.into_value()),+])
            }
        }

        impl <$($R),+> IntoValue<types::List<types::Any>> for ($($R),+)
        where $($R: IntoValue<types::Any>),+
        {
            fn into_value(self) -> Value {
                Value::raw_collection(vec![$(self.$index.into_value()),+])
            }
        }

        impl <$($R),+> From<($($R),+)> for query::QueryValues
        where $($R: IntoValue<types::Any>),+
        {
            fn from(tuple: ($($R),+)) -> Self {
                query::QueryValues(vec![$(tuple.$index.into_value()),+])
            }
        }

        impl<$($R),+> DefaultGrpcType for ($($R),+)
        where $($R: DefaultGrpcType),+
        {
            type C = ($(<$R as DefaultGrpcType>::C),+);
        }


    }
}

// Unfortunately I haven't figured out how to do all prefixes recursively with declarative macros.
// We could specify args in reversed order and generate all suffixes easily with one call,
// but then this would force us to process tuples right-to-left and the result vector would
// be also reversed (so additional reverse step would be needed in runtime to fix that).
// Hence, a bit verbose, but works:
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8, 9: R9 => C9);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8, 9: R9 => C9, 10: R10 => C10);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8, 9: R9 => C9, 10: R10 => C10, 11: R11 => C11);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8, 9: R9 => C9, 10: R10 => C10, 11: R11 => C11,
    12: R12 => C12);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8, 9: R9 => C9, 10: R10 => C10, 11: R11 => C11,
    12: R12 => C12, 13: R13 => C13);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8, 9: R9 => C9, 10: R10 => C10, 11: R11 => C11,
    12: R12 => C12, 13: R13 => C13, 14: R14 => C14);
gen_tuple_conversion!(
    0: R0 => C0, 1: R1 => C1, 2: R2 => C2, 3: R3 => C3,
    4: R4 => C4, 5: R5 => C5, 6: R6 => C6, 7: R7 => C7,
    8: R8 => C8, 9: R9 => C9, 10: R10 => C10, 11: R11 => C11,
    12: R12 => C12, 13: R13 => C13, 14: R14 => C14, 15: R15 => C15);

impl<R, C> IntoValue<C> for Option<R>
where
    R: IntoValue<C>,
    C: ConcreteType,
{
    fn into_value(self) -> Value {
        match self {
            None => Value::null(),
            Some(v) => v.into_value(),
        }
    }
}

impl<R, C> IntoValue<types::List<C>> for Vec<R>
where
    R: IntoValue<C>,
{
    fn into_value(self) -> Value {
        let elements = self.into_iter().map(|e| e.into_value()).collect_vec();
        Value::raw_collection(elements)
    }
}

impl<RK, RV, CK, CV> IntoValue<types::Map<CK, CV>> for Vec<(RK, RV)>
where
    RK: IntoValue<CK>,
    RV: IntoValue<CV>,
{
    fn into_value(self) -> Value {
        let elements = self
            .into_iter()
            .map(|(k, v)| (k.into_value(), v.into_value()));
        Value::map(elements)
    }
}

impl<RK, RV, CK, CV> IntoValue<types::Map<CK, CV>> for Vec<KeyValue<RK, RV>>
where
    RK: IntoValue<CK>,
    RV: IntoValue<CV>,
{
    fn into_value(self) -> Value {
        let elements = self
            .into_iter()
            .map(|KeyValue(k, v)| (k.into_value(), v.into_value()));
        Value::map(elements)
    }
}

impl<RK, RV, CK, CV> IntoValue<types::Map<CK, CV>> for BTreeMap<RK, RV>
where
    RK: IntoValue<CK> + Ord,
    RV: IntoValue<CV>,
{
    fn into_value(self) -> Value {
        let elements = self
            .into_iter()
            .map(|(k, v)| (k.into_value(), v.into_value()));
        Value::map(elements)
    }
}

impl<RK, RV, CK, CV> IntoValue<types::Map<CK, CV>> for HashMap<RK, RV>
where
    RK: IntoValue<CK> + Eq + Hash,
    RV: IntoValue<CV>,
{
    fn into_value(self) -> Value {
        let elements = self
            .into_iter()
            .map(|(k, v)| (k.into_value(), v.into_value()));
        Value::map(elements)
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> IntoValue<types::Int> for chrono::DateTime<Tz> {
    fn into_value(self) -> Value {
        Value::raw_int(self.timestamp_millis())
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> IntoValue<types::Date> for chrono::Date<Tz> {
    fn into_value(self) -> Value {
        use chrono::Datelike;
        Value::raw_date(self.num_days_from_ce() as u32)
    }
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::time::{SystemTime, UNIX_EPOCH};

    use proto::value::Inner;

    use crate::types::{Any, Date, Int, List, Map, Time};
    use crate::*;

    #[test]
    fn convert_value_into_value() {
        let v: Value = Value::int(1).into();
        assert_eq!(v, Value::int(1));
    }

    #[test]
    fn convert_i64_into_any_using_of_type() {
        let v: Value = Value::of_type(Any, 1);
        assert_eq!(v, Value::int(1));
    }

    #[test]
    fn convert_value_into_any_using_of_type() {
        let v: Value = Value::of_type(Any, Value::int(1));
        assert_eq!(v, Value::int(1))
    }

    #[test]
    fn convert_i64_into_value() {
        let v: Value = 100.into();
        assert_eq!(v, Value::int(100));
    }

    #[test]
    fn convert_float_into_value() {
        let v: Value = 100.0f32.into();
        assert_eq!(v, Value::float(100.0));
    }

    #[test]
    fn convert_double_into_value() {
        let v: Value = 100.0.into();
        assert_eq!(v, Value::double(100.0));
    }

    #[test]
    fn convert_string_into_value() {
        let v: Value = "foo".into();
        assert_eq!(v, Value::string("foo"));

        let v: Value = "foo".to_string().into();
        assert_eq!(v, Value::string("foo"));
    }

    #[test]
    fn convert_vector_into_bytes_value() {
        let buf: Vec<u8> = vec![1, 2];
        let v = Value::from(buf);
        assert_eq!(v, Value::bytes(vec![1, 2]))
    }

    #[test]
    fn convert_uuid_into_value() {
        let uuid = proto::Uuid { value: vec![1; 16] };
        let v = Value::from(uuid);
        assert_eq!(v, Value::uuid(&[1; 16]))
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn convert_uuid_uuid_into_value() {
        let uuid = uuid::Uuid::new_v4();
        let v1 = Value::from(uuid);
        let v2 = Value::uuid(uuid);
        assert_eq!(v1, v2)
    }

    #[test]
    fn convert_inet_into_value() {
        let inet = proto::Inet {
            value: vec![127, 0, 0, 1],
        };
        let v = Value::from(inet);
        assert_eq!(v, Value::inet(&[127, 0, 0, 1]))
    }

    #[test]
    fn convert_decimal_into_value() {
        let decimal = proto::Decimal {
            scale: 2,
            value: vec![10, 0],
        };
        let v = Value::from(decimal);
        assert_eq!(v, Value::raw_decimal(2, vec![10, 0]))
    }

    #[test]
    fn convert_varint_into_value() {
        let varint = proto::Varint { value: vec![10, 0] };
        let v = Value::from(varint);
        assert_eq!(v, Value::varint(vec![10, 0]))
    }

    #[test]
    fn convert_system_time_into_value() {
        let time = SystemTime::now();
        let unix_time = time.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let value1 = Value::from(time);
        assert_eq!(value1, Value::int(unix_time));
        let value2 = Value::int(time);
        assert_eq!(value2, Value::int(unix_time));
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn convert_chrono_utc_time_into_value() {
        let time = chrono::Utc::now();
        let unix_time = time.timestamp_millis() as i64;
        let value1 = Value::from(time);
        assert_eq!(value1, Value::int(unix_time));
        let value2 = Value::int(time);
        assert_eq!(value2, Value::int(unix_time));
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn convert_chrono_local_time_into_value() {
        let time = chrono::Local::now();
        let unix_time = time.timestamp_millis() as i64;
        let value = Value::from(time);
        assert_eq!(value, Value::int(unix_time));
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn convert_chrono_utc_date_into_value() {
        use chrono::Datelike;
        let date = chrono::Utc::now().date();
        let days = date.num_days_from_ce() as u32;
        let value = Value::from(date);
        assert_eq!(value, Value::date(days));
        let value = Value::date(date);
        assert_eq!(value, Value::date(days));
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn convert_chrono_local_date_into_value() {
        use chrono::Datelike;
        let date = chrono::Local::now().date();
        let days = date.num_days_from_ce() as u32;
        let value = Value::from(date);
        assert_eq!(value, Value::date(days));
    }

    #[test]
    fn convert_tuple_into_default_value() {
        let tuple = (1, "foo");
        let v = Value::from(tuple);
        assert_eq!(v, Value::list(vec![Value::int(1), Value::string("foo")]))
    }

    #[test]
    fn convert_tuple_into_list_value_using_of_type() {
        let tuple = (1, "foo");
        let v = Value::of_type(List(Any), tuple);
        assert_eq!(v, Value::list(vec![Value::int(1), Value::string("foo")]))
    }

    #[test]
    fn convert_tuple_into_typed_value() {
        let tuple = (1, 100);
        let v = Value::of_type((Int, Time), tuple);
        assert_eq!(v, Value::list(vec![Value::int(1), Value::time(100)]))
    }

    #[test]
    fn convert_large_tuple_into_value() {
        let tuple = (1, 2, 3, 4, 5, "foo");
        let v = Value::from(tuple);
        match v.inner {
            Some(Inner::Collection(value)) if value.elements.len() == 6 => {}
            inner => assert!(false, "Unexpected inner value {:?}", inner),
        }
    }

    #[test]
    fn convert_option_into_value() {
        let some: Option<i64> = Some(123);
        let some_value: Value = some.into();
        assert_eq!(Value::int(123), some_value);

        let none: Option<i64> = None;
        let none_value: Value = none.into();
        assert_eq!(Value::null(), none_value);
    }

    #[test]
    fn convert_option_into_any_using_of_type() {
        let v: Value = Value::of_type(Any, Some(1));
        assert_eq!(v, Value::int(1));

        let v: Value = Value::of_type(Any, None as Option<i32>);
        assert_eq!(v, Value::null());
    }

    #[test]
    fn convert_vec_of_i64_into_value() {
        let list = vec![1, 2];
        let v1 = Value::from(list.clone());
        let v2 = Value::list(list.clone());
        assert_eq!(v1, Value::list(vec![Value::int(1), Value::int(2)]));
        assert_eq!(v1, v2);
    }

    #[test]
    fn convert_nested_vec_i64_into_value() {
        let list = vec![vec![1, 2]];
        let expected = Value::list(vec![Value::list(vec![Value::int(1), Value::int(2)])]);
        let converted = Value::from(list);
        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_vec_of_dates_into_value() {
        let list = vec![1, 2];
        let v = Value::of_type(List(Date), list);
        assert_eq!(v, Value::list(vec![Value::date(1), Value::date(2)]));
    }

    #[test]
    fn convert_vec_of_pairs_into_map_value() {
        let expected = Value::map(vec![
            (Value::int(1), Value::string("foo")),
            (Value::int(2), Value::string("bar")),
        ]);

        let list1 = vec![KeyValue(1, "foo"), KeyValue(2, "bar")];
        assert_eq!(Value::from(list1), expected.clone());

        let list2 = vec![(1, "foo"), (2, "bar")];
        assert_eq!(
            Value::of_type(Map(Int, types::String), list2),
            expected.clone()
        );
    }

    #[test]
    fn convert_btree_map_into_value() {
        let mut map = BTreeMap::new();
        map.insert(1, "foo");
        map.insert(2, "bar");

        assert_eq!(
            Value::from(map),
            Value::map(vec![
                (Value::int(1), Value::string("foo")),
                (Value::int(2), Value::string("bar")),
            ])
        );
    }

    #[test]
    fn convert_hash_map_into_value() {
        let mut map = HashMap::new();
        map.insert(1, "foo"); // insert just one, so we don't run into problems with order

        assert_eq!(
            Value::from(map),
            Value::map(vec![(Value::int(1), Value::string("foo"))])
        );
    }

    #[test]
    fn convert_hash_map_to_udt_value() {
        let mut map = HashMap::new();
        map.insert("field1", Value::int(1));
        map.insert("field2", Value::string("bar"));
        let v = Value::udt(map);
        match v.inner {
            Some(Inner::Udt(value)) if value.fields.len() == 2 => {}
            inner => assert!(false, "Unexpected udt inner value {:?}", inner),
        }
    }

    #[test]
    fn convert_raw_udt_value_to_value() {
        let mut map = HashMap::new();
        map.insert("field1".to_string(), Value::int(1));
        map.insert("field2".to_string(), Value::string("bar"));
        let udt_value = proto::UdtValue { fields: map };
        let v = Value::from(udt_value);
        match v.inner {
            Some(Inner::Udt(value)) if value.fields.len() == 2 => {}
            inner => assert!(false, "Unexpected udt inner value {:?}", inner),
        }
    }
}
