//! # Automatic conversions from `Value` to standard Rust types.
//!
//! You can convert a `Value` into a Rust type by calling [`Value::try_into`].
//!
//! Because a `Value` can hold one of many different types, a conversion to a concrete
//! Rust type may fail with [`ConversionError`] if the actual runtime type of the
//! value is incompatible.
//!
//! ```
//! # use stargate_grpc::error::ConversionError;
//! use stargate_grpc::Value;
//!
//! let int: i64 = Value::int(10).try_into()?;
//! let string: String = Value::string("foo").try_into()?;
//! let list: Vec<i64> = Value::list(vec![Value::int(1), Value::int(2)]).try_into()?;
//! let (a, b): (i64, f64) = Value::list(vec![Value::int(1), Value::double(3.14)]).try_into()?;
//!
//! # Ok::<(), ConversionError>(())
//! ```
//!
//! ## Available conversions
//!
//! gRPC variant  |  Rust types
//! --------------| --------------------------------------------
//! `Boolean`     | `bool`
//! `Bytes`       | `Vec<u8>`
//! `Inet`        | [`proto::Inet`]
//! `Int`         | `i64`, `std::time::SystemTime`,`chrono::DateTime<Local>`, `chrono::DateTime<Utc>`
//! `Double`      | `f64`
//! `Date`        | `u32`, `chrono::Date<Local>`, `chrono::Date<Utc>`
//! `Decimal`     | [`proto::Decimal`]
//! `Float`       | `f32`
//! `String`      | `String`
//! `Time`        | `u64`
//! `Uuid`        | [`proto::Uuid`], `uuid::Uuid`
//! `Udt`         | [`proto::UdtValue`]
//! `Varint`      | [`proto::Varint`]
//! `Collection`  | `Vec<T>`, `HashMap<K, V>`, `BTreeMap<K, V>`, `(T1, T2, ..., Tn)`
//!
//! ## Handling nulls
//!
//! A `Value` can be a `null` or `unset`. If you try to convert a
//! `null` or `unset` value to a non-optional Rust type that can't represent a "lack of value", a
//! `ConversionError` of `ConversionErrorKind::Incompatible` will be returned.
//!
//! If you expect nulls, wrap your target type into an `Option`:
//! ```no_run
//! # use stargate_grpc::Value;
//! # use stargate_grpc::error::ConversionError;
//! let opt_int: Option<i64> = Value::null().try_into()?;  // ok
//! let int: i64 = Value::null().try_into()?;              // would fail with ConversionError
//! # Ok::<(), ConversionError>(())
//! ```
//!
//! ## Converting to `chrono::Date` and `chrono::DateTime`
//!
//! In order to be able to convert `Value`s into `chrono` dates and timestamps,
//! add `chrono` crate to dependencies of your project and enable `chrono` feature on this crate.
//! This crate can create only date and time values in the `chrono::Local` and `chrono::Utc`
//! timezones.
//!
//! ```rust
//! # use stargate_grpc::error::ConversionError;
//! # use stargate_grpc::Value;
//! # #[cfg(feature = "chrono")] {
//! use chrono::{DateTime, Utc};
//! let timestamp: DateTime<Utc> = Value::int(1633478400021_i64).try_into()?;
//! assert_eq!(timestamp.to_rfc2822(), "Wed, 06 Oct 2021 00:00:00 +0000");
//! # }
//! # Ok::<(), ConversionError>(())
//! ```
//!
//! ## Converting to `uuid::Uuid`
//! Similarly a `Value` of UUID type can be converted to `uuid::Uuid` once you enable feature
//! `uuid`.
//!
//! ```rust
//! # use stargate_grpc::error::ConversionError;
//! # use stargate_grpc::Value;
//! # #[cfg(feature = "uuid")] {
//! let uuid = 0x69415263_a826_4bd0_a0c16558c400d084_u128.to_le_bytes();
//! let uuid: uuid::Uuid = Value::raw_uuid(&uuid).try_into()?;
//! # }
//! # Ok::<(), ConversionError>(())
//! ```
//!
//! ## Custom conversions
//! You can make `Value` convertible to any type by implementing the [`TryFromValue`] trait.
//!
//! For example, let's define such conversion into a custom `Login` struct that wraps a `String`:
//! ```
//! use stargate_grpc::error::ConversionError;
//! use stargate_grpc::from_value::TryFromValue;
//! use stargate_grpc::Value;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Login(String);
//!
//! impl TryFromValue for Login {
//!     fn try_from(value: Value) -> Result<Self, ConversionError> {
//!         Ok(Login(String::try_from(value)?))
//!     }
//! }
//!
//! let login: Login = Value::string("login").try_into()?;
//! assert_eq!(login, Login("login".to_string()));
//! # Ok::<(), ConversionError>(())
//! ```


use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::error::Error;
use std::hash::Hash;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use itertools::Itertools;

use crate::error::ConversionError;
use crate::proto::{value, Row, Value};
use crate::{proto, KeyValue};

/// Converts a `Value` to a Rust type.
///
/// Implementations are provided for most commonly used Rust types.
/// Implementations must not cause silent precision loss -
/// e.g. converting from a `Double` to `f32` is not allowed.
/// Returns `ConversionError` if the `Value` variant is incompatible with the target Rust type.
/// A `ConversionError` is also returned if the underlying value is `Null` or `Unset`, but
/// the receiving type can't handle nulls, i.e. it is not a `Value` nor `Option`.
///
/// We are not using the `TryFrom` trait from Rust core directly, because Rust stdlib defines
/// blanket implementations of `TryFrom` and `TryInto` which would conflict with
/// the implementations of this trait for converting e.g. `Value` into an `Option<T>`.
/// Instead we selectively generate `TryFrom` implementations from `TryFromValue`
/// using dedicated macros.
pub trait TryFromValue: Sized {
    fn try_from(value: Value) -> Result<Self, ConversionError>;
}

impl Value {
    pub fn try_into<T: TryFromValue>(self) -> Result<T, ConversionError> {
        T::try_from(self)
    }
}

impl Error for ConversionError {}

/// Generates the implementation of `TryFrom<Value>` for a concrete type `T` given as argument.
/// The conversion is delegated to `TryFromValue` trait that must be defined for `T`.
macro_rules! gen_std_conversion {
    ($T:ty) => {
        impl TryFrom<Value> for $T {
            type Error = ConversionError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                value.try_into()
            }
        }
    };
}

/// Same as `gen_std_conversion` but accepts generic types.
///
/// The macro syntax is: `gen_try_from_generic!(<Arg1, Arg2, ..., ArgN> GenericType)`.
/// All type arguments must have implementations of `TryFromValue`.
/// Type arguments are allowed to define additional type bounds, using standard Rust syntax.
macro_rules! gen_std_conversion_generic {
    (<$($A:ident $(: $bound_1:tt $( +$bound_n:tt )* )?),+> $T:ty) => {
        impl<$($A),+> TryFrom<Value> for $T
        where $($A: TryFromValue $(+ $bound_1 $(+ $bound_n)* )?),+
        {
            type Error = ConversionError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                value.try_into()
            }
        }
    };
}

/// Generates a `TryFromValue` for given concrete Rust type.
macro_rules! gen_conversion {
    ($T:ty; $( $from:pat_param => $to:expr ),+) => {

        impl TryFromValue for $T {
            fn try_from(value: Value) -> Result<Self, ConversionError> {
                match value.inner {
                    $(Some($from) => $to)+,
                    other => Err(ConversionError::incompatible::<_, Self>(other)),
                }
            }
        }

        gen_std_conversion!($T);
        gen_std_conversion!(Option<$T>);
    }
}

gen_conversion!(bool; value::Inner::Boolean(x) => Ok(x));
gen_conversion!(i64; value::Inner::Int(x) => Ok(x));
gen_conversion!(u32; value::Inner::Date(x) => Ok(x));
gen_conversion!(u64; value::Inner::Time(x) => Ok(x));
gen_conversion!(f32; value::Inner::Float(x) => Ok(x));
gen_conversion!(f64; value::Inner::Double(x) => Ok(x));
gen_conversion!(String; value::Inner::String(x) => Ok(x));
gen_conversion!(Vec<u8>; value::Inner::Bytes(x) => Ok(x));

gen_conversion!(proto::Decimal; value::Inner::Decimal(x) => Ok(x));
gen_conversion!(proto::Inet; value::Inner::Inet(x) => Ok(x));
gen_conversion!(proto::UdtValue; value::Inner::Udt(x) => Ok(x));
gen_conversion!(proto::Uuid; value::Inner::Uuid(x) => Ok(x));
gen_conversion!(proto::Varint; value::Inner::Varint(x) => Ok(x));

#[cfg(feature = "uuid")]
gen_conversion!(uuid::Uuid; value::Inner::Uuid(x) =>
    uuid::Uuid::from_slice(&x.value)
        .map_err(|_| {
            let actual_len = x.value.len();
            ConversionError::wrong_number_of_items::<_, uuid::Uuid>(x, actual_len, 16)
        })
);

gen_conversion!(SystemTime; value::Inner::Int(ts) => {
    Ok(UNIX_EPOCH.checked_add(Duration::from_millis(ts as u64)).unwrap())
});

#[cfg(feature = "chrono")]
gen_conversion!(chrono::DateTime<chrono::Utc>; value::Inner::Int(millis) => {
    use chrono::TimeZone;
    Ok(chrono::Utc.timestamp(
        millis.div_euclid(1000) as i64,
        (millis.rem_euclid(1000) * 1_000_000) as u32
    ))
});

#[cfg(feature = "chrono")]
gen_conversion!(chrono::DateTime<chrono::Local>; value::Inner::Int(millis) => {
    use chrono::TimeZone;
    Ok(chrono::Local.timestamp(
        millis.div_euclid(1000) as i64,
        (millis.rem_euclid(1000) * 1_000_000) as u32
    ))
});

#[cfg(feature = "chrono")]
fn into_naive_date(days: u32) -> Result<chrono::NaiveDate, ConversionError> {
    use std::convert::TryInto;
    let err = || ConversionError::out_of_range::<_, chrono::Date<chrono::Local>>(days);
    let days: i32 = days.try_into().map_err(|_| err())?;
    chrono::NaiveDate::from_num_days_from_ce_opt(days).ok_or_else(err)
}

#[cfg(feature = "chrono")]
gen_conversion!(chrono::Date<chrono::Utc>; value::Inner::Date(days) => {
    use chrono::TimeZone;
    Ok(chrono::Utc.from_utc_date(&into_naive_date(days)?))
});

#[cfg(feature = "chrono")]
gen_conversion!(chrono::Date<chrono::Local>; value::Inner::Date(days) => {
    use chrono::TimeZone;
    Ok(chrono::Local.from_utc_date(&into_naive_date(days)?))
});

/// Counts the number of arguments
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

/// Generates `TryFromValue`, `TryFrom<Value>` and `TryFrom<Row>`
/// implementations for tuples of fixed size, denoted by the number of arguments.
macro_rules! gen_tuple_conversion {
    ($($T:ident),+) => {

        // Converts values to tuples
        // E.g. for 2-ary tuples expands to: `impl <A2, A1> TryFromValue for (A2, A1)`
        impl<$($T),+> TryFromValue for ($($T),+)
        where $($T: TryFromValue),+
        {
            fn try_from(value: Value) -> Result<Self, ConversionError> {
                match value.inner {
                    // if the size doesn't match, we just bail out in the `other` case
                    Some(value::Inner::Collection(c)) => {
                        let len = c.elements.len();
                        let expected_len = count!($($T)+);
                        if len != expected_len {
                            return Err(ConversionError::wrong_number_of_items::<_, Self>(c, len, expected_len));
                        }
                        let mut i = c.elements.into_iter();
                        Ok((
                            $({ let x: $T = i.next().unwrap().try_into()?; x }),+
                        ))
                    }
                    other => Err(ConversionError::incompatible::<_, Self>(other)),
                }
            }
        }

        // Generates an analog `TryFrom<Value>` for tuples.
        // for 2-ary tuples expands to: `gen_std_conversion_generic!(<A2, A1> (A2, A1))`
        gen_std_conversion_generic!(<$($T),+> ($($T),+));

        // Converts rows to tuples
        impl<$($T),+> TryFrom<Row> for ($($T),+)
        where $($T: TryFromValue),+
        {
            type Error = ConversionError;

            fn try_from(row: Row) -> Result<Self, ConversionError> {
                let len = row.values.len();
                let expected_len = count!($($T)+);
                if len != expected_len {
                    return Err(ConversionError::wrong_number_of_items::<_, Self>(row, len, expected_len));
                }
                let mut i = row.values.into_iter();
                Ok((
                    $({ let x: $T = i.next().unwrap().try_into()?; x }),+
                ))
            }
        }
    }
}

/// Calls `gen_convert_value_tuple!` recursively to generate conversions for all tuples
/// starting at size 2 and ending at the size specified by the number of arguments.
macro_rules! gen_all_tuple_conversions {
    ($first:ident) => {};
    ($first:ident, $($tail:ident),*) => {
        gen_tuple_conversion!($first, $($tail),*);
        gen_all_tuple_conversions!($($tail),*);
    }
}

// Generate conversions for all tuples up to size 16
gen_all_tuple_conversions!(A16, A15, A14, A13, A12, A11, A10, A9, A8, A7, A6, A5, A4, A3, A2, A1);

/// Converts the Value into itself.
/// Actually the compiler will translate it to a no-op, as no copies are made.
///
/// This conversion is needed in order to be able to convert a `Value` representing a collection
/// into a `Vec<Value>` without converting the elements of the collection. You may want to
/// leave the elements unconverted, if they are of different types (heterogeneous collection).
impl TryFromValue for Value {
    fn try_from(value: Value) -> Result<Self, ConversionError> {
        Ok(value)
    }
}

/// Converts CQL `Nulls` and `Unset` to `None`.
/// Note that if a value exists, but is of an unexpected type, an `ConversionError` is returned.
impl<T> TryFromValue for Option<T>
where
    T: TryFromValue,
{
    fn try_from(value: Value) -> Result<Self, ConversionError> {
        match &value.inner {
            None => Ok(None),
            Some(value::Inner::Null(_)) => Ok(None),
            Some(value::Inner::Unset(_)) => Ok(None),
            Some(_) => Ok(Some(value.try_into()?)),
        }
    }
}

/// Converts a `Value` into a vector, converting all elements to appropriate type `T` if needed.
/// `T` can be any type that have a supported conversion from `Value`.
/// It is also allowed that `T == Value` so you can get a heterogeneous collection back.
impl<T> TryFromValue for Vec<T>
where
    T: TryFromValue,
{
    fn try_from(value: Value) -> Result<Self, ConversionError> {
        match value.inner {
            Some(value::Inner::Collection(c)) => {
                Ok(c.elements.into_iter().map(|e| e.try_into()).try_collect()?)
            }
            other => Err(ConversionError::incompatible::<_, Self>(other)),
        }
    }
}

/// Converts a `Value` representing a map into a vector of key-value pairs.
/// Order of the items is the same as received from the server.
impl<K, V> TryFromValue for Vec<KeyValue<K, V>>
where
    K: TryFromValue,
    V: TryFromValue,
{
    fn try_from(value: Value) -> Result<Self, ConversionError> {
        match value.inner {
            Some(value::Inner::Collection(c)) if c.elements.len() % 2 == 0 => {
                let mut result = Vec::with_capacity(c.elements.len() / 2);
                for (k, v) in c.elements.into_iter().tuples() {
                    let k: K = k.try_into()?;
                    let v: V = v.try_into()?;
                    result.push(KeyValue(k, v));
                }
                Ok(result)
            }
            other => Err(ConversionError::incompatible::<_, Self>(other)),
        }
    }
}

/// Converts a `Value` representing a map into a hash-map.
/// Obviously the order is undefined
impl<K, V> TryFromValue for HashMap<K, V>
where
    K: TryFromValue + Eq + Hash,
    V: TryFromValue,
{
    fn try_from(value: Value) -> Result<Self, ConversionError> {
        let pairs: Vec<KeyValue<K, V>> = value.try_into()?;
        let mut map = HashMap::with_capacity(pairs.len());
        map.extend(pairs.into_iter().map(|kv| kv.into_tuple()));
        Ok(map)
    }
}

impl<K, V> TryFromValue for BTreeMap<K, V>
where
    K: TryFromValue + Ord,
    V: TryFromValue,
{
    fn try_from(value: Value) -> Result<Self, ConversionError> {
        let pairs: Vec<KeyValue<K, V>> = value.try_into()?;
        let mut map = BTreeMap::new();
        map.extend(pairs.into_iter().map(|kv| kv.into_tuple()));
        Ok(map)
    }
}

gen_std_conversion_generic!(<T> Vec<T>);
gen_std_conversion_generic!(<T> Option<Vec<T>>);
gen_std_conversion_generic!(<K, V> Vec<KeyValue<K, V>>);
gen_std_conversion_generic!(<K, V> Option<Vec<KeyValue<K, V>>>);
gen_std_conversion_generic!(<K: Eq + Hash, V> HashMap<K, V>);
gen_std_conversion_generic!(<K: Eq + Hash, V> Option<HashMap<K, V>>);
gen_std_conversion_generic!(<K: Ord, V> BTreeMap<K, V>);
gen_std_conversion_generic!(<K: Ord, V> Option<BTreeMap<K, V>>);

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn convert_value_to_i64() {
        let v = Value::int(123);
        let int: i64 = v.try_into().unwrap();
        assert_eq!(int, 123)
    }

    #[test]
    fn convert_value_to_f32() {
        let v = Value::float(3.5);
        let float: f32 = v.try_into().unwrap();
        assert_eq!(float, 3.5)
    }

    #[test]
    fn convert_value_to_f64() {
        let v = Value::double(3.5);
        let double: f64 = v.try_into().unwrap();
        assert_eq!(double, 3.5)
    }

    #[test]
    fn convert_value_to_string() {
        let v = Value::string("foo");
        let s: String = v.try_into().unwrap();
        assert_eq!(s, "foo".to_string())
    }

    #[test]
    fn convert_bytes_value_to_vec() {
        let v = Value::bytes(vec![1, 2]);
        let buf: Vec<u8> = v.try_into().unwrap();
        assert_eq!(buf, vec![1, 2])
    }

    #[test]
    fn convert_value_to_inet() {
        let v = Value::raw_inet(vec![1, 2]);
        let inet: proto::Inet = v.try_into().unwrap();
        assert_eq!(inet, proto::Inet { value: vec![1, 2] })
    }

    #[test]
    fn convert_value_to_decimal() {
        let v = Value::raw_decimal(2, vec![1, 2]);
        let decimal: proto::Decimal = v.try_into().unwrap();
        assert_eq!(
            decimal,
            proto::Decimal {
                scale: 2,
                value: vec![1, 2]
            }
        )
    }

    #[test]
    fn convert_value_to_varint() {
        let v = Value::raw_varint(vec![1, 2]);
        let varint: proto::Varint = v.try_into().unwrap();
        assert_eq!(varint, proto::Varint { value: vec![1, 2] })
    }

    #[test]
    fn convert_value_to_uuid() {
        let v = Value::raw_uuid(&[1; 16]);
        let uuid: proto::Uuid = v.try_into().unwrap();
        assert_eq!(
            uuid,
            proto::Uuid {
                value: [1; 16].to_vec()
            }
        )
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn convert_value_to_uuid_uuid() {
        let v = Value::raw_uuid(&[1; 16]);
        let uuid: uuid::Uuid = v.try_into().unwrap();
        assert_eq!(uuid.as_bytes(), &[1; 16])
    }

    #[test]
    fn convert_value_to_system_time() {
        let v = Value::int(10000);
        let time: SystemTime = v.try_into().unwrap();
        assert_eq!(time.duration_since(UNIX_EPOCH).unwrap().as_millis(), 10000);
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn convert_value_to_chrono_date_time() {
        let v = Value::int(10000);
        let time: chrono::DateTime<chrono::Utc> = v.try_into().unwrap();
        assert_eq!(time.timestamp_millis(), 10000);
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn convert_value_to_chrono_date() {
        use chrono::Datelike;
        let v = Value::date(10000);
        let date: chrono::Date<chrono::Utc> = v.try_into().unwrap();
        assert_eq!(date.num_days_from_ce(), 10000);
    }

    #[test]
    fn convert_value_to_option() {
        let some = Value::int(123);
        let none = Value::null();

        let some_int: Option<i64> = some.try_into().unwrap();
        let none_int: Option<i64> = none.try_into().unwrap();

        assert_eq!(some_int, Some(123));
        assert_eq!(none_int, None);
    }

    #[test]
    fn convert_value_to_heterogeneous_vec() {
        let v1 = Value::int(1);
        let v2 = Value::int(2);
        let v = Value::list(vec![v1.clone(), v2.clone()]);

        let vec: Vec<Value> = v.try_into().unwrap();
        assert_eq!(vec, vec![v1, v2]);
    }

    #[test]
    fn convert_value_to_homogenous_vec() {
        let v1 = Value::int(1);
        let v2 = Value::int(2);
        let v = Value::list(vec![v1, v2]);

        let vec: Vec<i64> = v.try_into().unwrap();
        assert_eq!(vec, vec![1, 2]);
    }

    #[test]
    fn convert_value_to_vec_of_key_value() {
        let v1 = Value::int(1);
        let v2 = Value::int(2);
        let v = Value::list(vec![v1, v2]);
        let vec: Vec<KeyValue<i64, i64>> = v.try_into().unwrap();
        assert_eq!(vec, vec![KeyValue(1, 2)]);
    }

    #[test]
    fn convert_value_to_hash_map() {
        let v1 = Value::int(1);
        let v2 = Value::string("foo".to_string());
        let v = Value::list(vec![v1, v2]);
        let map: HashMap<i64, String> = v.try_into().unwrap();
        assert_eq!(map.get(&1), Some("foo".to_string()).as_ref());
    }

    #[test]
    fn convert_value_to_btree_map() {
        let v1 = Value::int(1);
        let v2 = Value::string("foo".to_string());
        let v = Value::list(vec![v1, v2]);
        let map: BTreeMap<i64, String> = v.try_into().unwrap();
        assert_eq!(map.get(&1), Some("foo".to_string()).as_ref());
    }

    #[test]
    fn convert_value_to_nested_collections() {
        let key = Value::string("foo".to_string());
        let v1 = Value::int(1);
        let v2 = Value::int(2);
        let list = Value::list(vec![v1, v2]);
        let v = Value::list(vec![key, list]);

        let map: HashMap<String, Vec<i64>> = v.try_into().unwrap();
        assert_eq!(map.get(&"foo".to_string()), Some(vec![1, 2]).as_ref());
    }

    #[test]
    fn convert_value_to_tuples() {
        let v1 = Value::int(1);
        let v2 = Value::float(2.5);
        let v = Value::list(vec![v1, v2]);
        let (a, b): (i64, f32) = v.try_into().unwrap();
        assert_eq!(a, 1);
        assert_eq!(b, 2.5);
    }

    #[test]
    fn convert_value_to_triples() {
        let v1 = Value::int(1);
        let v2 = Value::int(2);
        let v3 = Value::float(2.5);
        let v = Value::list(vec![v1, v2, v3]);

        let (a, b, c): (i64, i64, f32) = v.try_into().unwrap();
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, 2.5);
    }

    #[test]
    fn unexpected_type() {
        let v = Value::int(123);
        assert!(v.try_into::<String>().is_err());
    }

    #[test]
    fn unexpected_tuple_size() {
        let v1 = Value::int(1);
        let v2 = Value::float(2.5);
        let v = Value::list(vec![v1, v2]);
        assert!(v.try_into::<(i64, f32, f32, f32)>().is_err());
    }

    #[test]
    fn pass_value_as_try_into() {
        fn into_i64<T: TryInto<i64>>(value: T) -> i64 {
            value.try_into().unwrap_or(-1)
        }

        let v1 = Value::int(1);
        assert_eq!(1, into_i64(v1));
    }

    #[test]
    fn convert_row_to_i64() {
        let values = vec![Value::int(1)];
        let row = Row { values };
        let int: i64 = row.try_into_one().unwrap();
        assert_eq!(int, 1);
    }

    #[test]
    fn convert_row_to_list() {
        let values = vec![Value::list(vec![1, 2, 3])];
        let row = Row { values };
        let int: Vec<i64> = row.try_into_one().unwrap();
        assert_eq!(int, vec![1, 2, 3]);
    }

    #[test]
    fn convert_row_to_tuple() {
        let values = vec![Value::int(1), Value::double(2.0), Value::string("foo")];
        let row = Row { values };
        let (a, b, c): (i64, f64, String) = row.try_into().unwrap();
        assert_eq!(a, 1);
        assert_eq!(b, 2.0);
        assert_eq!(c, "foo".to_string());
    }

    #[test]
    fn convert_single_item_of_a_row() {
        let values = vec![Value::int(1), Value::double(2.0), Value::string("foo")];
        let row = Row { values };
        let a: i64 = row.get(0).unwrap();
        let b: f64 = row.get(1).unwrap();
        let c: String = row.get(2).unwrap();
        assert_eq!(a, 1);
        assert_eq!(b, 2.0);
        assert_eq!(c, "foo".to_string());
    }
}
