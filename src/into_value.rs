//! Automatic conversions from standard Rust types to `Value`.
//!
//! Values can be obtained generically from commonly used Rust types using
//! standard Rust [`Into`](std::convert::Into) or [`From`](std::convert::From) traits:
//! ```rust
//! # use stargate_grpc::Value;
//!
//! let int_value: Value = 5.into();             // == Value::int(5)
//! let string_value: Value = "stargate".into(); // == Value::string("stargate")
//! let list1: Value = vec![1, 2].into();        // == Value::list(vec![Value::int(1), Value::int(2)])
//! let list2: Value = (1, 3.14).into();         // == Value::list(vec![Value::int(1), Value::double(3.14)])
//! ```
//!
//! It is also possible to specify the desired target gRPC type to use [`Value::of_type()`]
//! to disambiguate when more
//! target types are possible or to make the conversion more type-safe:
//! ```rust
//! # use stargate_grpc::{types, Value};
//!
//! let int_value = Value::of_type(types::Int, 5);
//! let timestamp_value = Value::of_type(types::Time, 1633005636085);
//! // let string_value = Value::of_type(types::String, 10); // compile time error
//! ```
//! ## Available Conversions
//! | Rust type                     | gRPC type
//! |-------------------------------|------------------------------------
//! | `i8`                          | [`types::Int`]                    |
//! | `i16`                         | [`types::Int`]                    |
//! | `i32`                         | [`types::Int`]                    |
//! | `i64`                         | [`types::Int`]                    |
//! | `u16`                         | [`types::Int`]                    |
//! | `u32`                         | [`types::Int`], [`types::Date`]   |
//! | `u64`                         | [`types::Time`]                   |
//! | `f32`                         | [`types::Float`]                  |
//! | `f64`                         | [`types::Double`]                 |
//! | `bool`                        | [`types::Boolean`]                |
//! | `String`                      | [`types::String`]                 |
//! | `&str`                        | [`types::String`]                 |
//! | `Vec<u8>`                     | [`types::Bytes`]                  |
//! | `Vec<T>`                      | [`types::List`]                   |
//! | `Vec<KeyValue>`               | [`types::Map`]                    |
//! | `HashMap<K, V>`               | [`types::Map`]                    |
//! | `BTreeMap<K, V>`              | [`types::Map`]                    |
//! | `(T1, T2, ...)`               | [`types::List`]                   |
//! | [`proto::Decimal`]            | [`types::Decimal`]                |
//! | [`proto::Inet`]               | [`types::Inet`]                   |
//! | [`proto::UdtValue`]           | [`types::Udt`]                    |
//! | [`proto::Uuid`]               | [`types::Uuid`]                   |
//! | [`proto::Varint`]             | [`types::Varint`]                 |

use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::hash::Hash;

use itertools::Itertools;

use crate::types::ConcreteType;
use crate::*;

/// Selects the default Cassandra gRPC value type associated with a Rust type.
/// The default type is used when a Rust value `x` is converted to `Value` by calling
/// `x.into()` or `Value::from(x)`.
///
/// In order to convert a Rust value to a non-default Cassandra type, or to convert
/// a Rust type that doesn't have a default conversion defined, use [`Value::of_type()`].
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
pub trait DefaultCassandraType {
    /// gRPC type, must be set to one of the types defined in the [`types`](crate::types) module.
    type C;
}

impl DefaultCassandraType for bool {
    type C = types::Boolean;
}
impl DefaultCassandraType for i8 {
    type C = types::Int;
}
impl DefaultCassandraType for i16 {
    type C = types::Int;
}
impl DefaultCassandraType for i32 {
    type C = types::Int;
}
impl DefaultCassandraType for i64 {
    type C = types::Int;
}
impl DefaultCassandraType for u16 {
    type C = types::Int;
}
impl DefaultCassandraType for u32 {
    type C = types::Int;
}
impl DefaultCassandraType for f32 {
    type C = types::Float;
}
impl DefaultCassandraType for f64 {
    type C = types::Double;
}
impl DefaultCassandraType for String {
    type C = types::String;
}
impl DefaultCassandraType for &str {
    type C = types::String;
}
impl DefaultCassandraType for Vec<u8> {
    type C = types::Bytes;
}
impl DefaultCassandraType for proto::Decimal {
    type C = types::Decimal;
}
impl DefaultCassandraType for proto::Inet {
    type C = types::Inet;
}
impl DefaultCassandraType for proto::UdtValue {
    type C = types::Udt;
}
impl DefaultCassandraType for proto::Uuid {
    type C = types::Uuid;
}
#[cfg(feature = "uuid")]
impl DefaultCassandraType for uuid::Uuid {
    type C = types::Uuid;
}

impl DefaultCassandraType for proto::Varint {
    type C = types::Varint;
}

impl<T> DefaultCassandraType for Option<T>
where
    T: DefaultCassandraType,
{
    type C = <T as DefaultCassandraType>::C;
}

impl<T> DefaultCassandraType for Vec<T>
where
    T: DefaultCassandraType,
{
    type C = types::List<<T as DefaultCassandraType>::C>;
}

impl<K, V> DefaultCassandraType for Vec<KeyValue<K, V>>
where
    K: DefaultCassandraType,
    V: DefaultCassandraType,
{
    type C = types::Map<<K as DefaultCassandraType>::C, <V as DefaultCassandraType>::C>;
}

impl<K, V> DefaultCassandraType for HashMap<K, V>
where
    K: DefaultCassandraType,
    V: DefaultCassandraType,
{
    type C = types::Map<<K as DefaultCassandraType>::C, <V as DefaultCassandraType>::C>;
}

impl<K, V> DefaultCassandraType for BTreeMap<K, V>
where
    K: DefaultCassandraType,
    V: DefaultCassandraType,
{
    type C = types::Map<<K as DefaultCassandraType>::C, <V as DefaultCassandraType>::C>;
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
    /// let timestamps = Value::of_type(List(Time), vec![1633005636085, 1633005636090]);
    /// assert_eq!(timestamps, Value::list(vec![
    ///     Value::time(1633005636085),
    ///     Value::time(1633005636090)
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

    /// Creates a Cassandra Null value.
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

    pub fn boolean(value: bool) -> Value {
        Value {
            inner: Some(proto::value::Inner::Boolean(value)),
        }
    }

    pub fn int(value: i64) -> Value {
        Value {
            inner: Some(proto::value::Inner::Int(value)),
        }
    }

    pub fn float(value: f32) -> Value {
        Value {
            inner: Some(proto::value::Inner::Float(value)),
        }
    }

    pub fn double(value: f64) -> Value {
        Value {
            inner: Some(proto::value::Inner::Double(value)),
        }
    }

    pub fn date(value: u32) -> Value {
        Value {
            inner: Some(proto::value::Inner::Date(value)),
        }
    }

    pub fn time(value: u64) -> Value {
        Value {
            inner: Some(proto::value::Inner::Time(value)),
        }
    }

    pub fn uuid(value: &[u8; 16]) -> Value {
        Value {
            inner: Some(proto::value::Inner::Uuid(proto::Uuid {
                value: value.to_vec(),
            })),
        }
    }

    pub fn inet(value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Inet(proto::Inet { value })),
        }
    }

    pub fn bytes(value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Bytes(value)),
        }
    }

    pub fn varint(value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Varint(proto::Varint { value })),
        }
    }

    pub fn decimal(scale: u32, value: Vec<u8>) -> Value {
        Value {
            inner: Some(proto::value::Inner::Decimal(proto::Decimal {
                scale,
                value,
            })),
        }
    }

    pub fn string<S: ToString>(value: S) -> Value {
        Value {
            inner: Some(proto::value::Inner::String(value.to_string())),
        }
    }

    /// Converts an iterable collection to a `Value` representing a list.
    /// Items are converted to `Value` using the default conversion.
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
    /// See also [`Value::list_of()`].
    pub fn list<I, T>(elements: I) -> Value
    where
        I: IntoIterator<Item = T>,
        T: Into<Value>,
    {
        Value::list_of(types::Any, elements)
    }

    /// Converts an iterable collection to a `Value` representing a list of elements of given type.
    /// Each element of the iterable is converted to a type denoted by type `E`.
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
        Value {
            inner: Some(proto::value::Inner::Collection(proto::Collection {
                elements,
            })),
        }
    }

    /// Converts a collection of key-value pairs to a `Value` representing a map.
    /// Keys and values of the map are converted to a `Value` of the default type.
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
    pub fn map<I, K, V>(key_value_pairs: I) -> Value
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Value>,
        V: Into<Value>,
    {
        Value::map_of(types::Any, types::Any, key_value_pairs)
    }

    /// Converts a collection of key-value pairs to a `Value` representing a Cassandra map.
    /// Keys and values of the map are converted to gRPC types specified by `CK` and `CV` types.
    ///
    /// The calling code will not compile if the elements of the map cannot be converted
    /// to given gRPC types.
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
    /// map.insert(1, Inet { value: vec![127, 0, 0, 1] });
    /// map.insert(2, Inet { value: vec![127, 0, 0, 2] });
    ///
    /// assert_eq!(
    ///     Value::map_of(types::Int, types::Inet, map),
    ///     Value::map(vec![
    ///         (Value::int(1), Value::inet(vec![127, 0, 0, 1])),
    ///         (Value::int(2), Value::inet(vec![127, 0, 0, 2]))
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

    /// Converts a collection of key-value pairs into a `Value` representing a Cassandra UDT.
    ///
    /// Keys must be convertible to strings anv values must be convertible to `Value`.
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
        Value::raw_udt(proto::UdtValue { fields })
    }

    fn raw_udt(value: proto::UdtValue) -> Value {
        Value {
            inner: Some(proto::value::Inner::Udt(value)),
        }
    }
}

impl<R> From<R> for Value
where
    R: DefaultCassandraType + IntoValue<<R as into_value::DefaultCassandraType>::C>,
{
    fn from(value: R) -> Self {
        Value::convert::<R, <R as DefaultCassandraType>::C>(value)
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

gen_conversion!(bool => types::Boolean; x => Value::boolean(x));

gen_conversion!(i64 => types::Int; x => Value::int(x));
gen_conversion!(i32 => types::Int; x => Value::int(x as i64));
gen_conversion!(i16 => types::Int; x => Value::int(x as i64));
gen_conversion!(i8 => types::Int; x => Value::int(x as i64));

//there is no u64 to Int conversion because it doesn't fit fully in the target range
gen_conversion!(u32 => types::Int; x => Value::int(x as i64));
gen_conversion!(u16 => types::Int; x => Value::int(x as i64));
gen_conversion!(u8 => types::Int; x => Value::int(x as i64));

gen_conversion!(u64 => types::Time; x => Value::time(x));
gen_conversion!(u32 => types::Date; x => Value::date(x));

gen_conversion!(f32 => types::Float; x => Value::float(x));
gen_conversion!(f64 => types::Double; x => Value::double(x));

gen_conversion!(String => types::String; x => Value::string(x));
gen_conversion!(&str => types::String; x => Value::string(x.to_string()));

gen_conversion!(Vec<u8> => types::Bytes; x => Value::bytes(x));

gen_conversion!(proto::Decimal => types::Decimal; x => Value::decimal(x.scale, x.value));
gen_conversion!(proto::Inet => types::Inet; x => Value::inet(x.value));
gen_conversion!(proto::UdtValue => types::Udt; x => Value::raw_udt(x));
gen_conversion!(proto::Uuid => types::Uuid; x => Value::uuid(&x.value.try_into().expect("16 bytes")));
gen_conversion!(proto::Varint => types::Varint; x => Value::varint(x.value));

#[cfg(feature = "uuid")]
gen_conversion!(uuid::Uuid => types::Uuid; x => Value::uuid(x.as_bytes()));

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
                Value::list(vec![$(self.$index.into_value()),+])
            }
        }

        impl <$($R),+> IntoValue<types::List<types::Any>> for ($($R),+)
        where $($R: IntoValue<types::Any>),+
        {
            fn into_value(self) -> Value {
                Value::list(vec![$(self.$index.into_value()),+])
            }
        }

        impl <$($R),+> From<($($R),+)> for query::QueryValues
        where $($R: IntoValue<types::Any>),+
        {
            fn from(tuple: ($($R),+)) -> Self {
                query::QueryValues(vec![$(tuple.$index.into_value()),+])
            }
        }

        impl<$($R),+> DefaultCassandraType for ($($R),+)
        where $($R: DefaultCassandraType),+
        {
            type C = ($(<$R as DefaultCassandraType>::C),+);
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
        Value::list(elements)
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

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};

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
    fn convert_inet_into_value() {
        let inet = proto::Inet {
            value: vec![127, 0, 0, 1],
        };
        let v = Value::from(inet);
        assert_eq!(v, Value::inet(vec![127, 0, 0, 1]))
    }

    #[test]
    fn convert_decimal_into_value() {
        let decimal = proto::Decimal {
            scale: 2,
            value: vec![10, 0],
        };
        let v = Value::from(decimal);
        assert_eq!(v, Value::decimal(2, vec![10, 0]))
    }

    #[test]
    fn convert_varint_into_value() {
        let varint = proto::Varint { value: vec![10, 0] };
        let v = Value::from(varint);
        assert_eq!(v, Value::varint(vec![10, 0]))
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
