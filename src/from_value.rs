//! Defines automatic conversions from `Value` to standard Rust types.

use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use itertools::Itertools;

use crate::*;

#[derive(Clone, Debug)]
pub struct ConversionError {
    value: String,
    rust_type_name: &'static str,
}

impl ConversionError {
    fn new<T, V: Debug>(cql_value: V) -> ConversionError {
        ConversionError {
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

/// Converts a `Value` to a Rust type.
///
/// Implementations are provided for most commonly used Rust types.
/// Implementations must not cause silent precision loss -
/// e.g. converting from a `Double` to `f32` is not allowed.
/// Returns `ConversionError` if the `Value` variant is incompatible with the target Rust type.
/// A `ConverrsionError` is also returned if the underlying value is `Null` or `Unset`, but
/// the receiving type can't handle nulls, i.e. it is not a `Value` nor `Option`.
///
/// We are not using the `TryFrom` trait from Rust core directly, because Rust stdlib defines
/// blanket implementations of `TryFrom` and `TryInto` which would conflict with
/// the implementations of this trait for converting e.g. `Value` into an `Option<T>`.
/// Instead we selectively generate `TryFrom` implementations from `TryFromValue`
/// using dedicated `gen_try_from` macros.
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

/// Same as `gen_try_from_for` but accepts generic types.
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
                    $(Some($from) => Ok($to)),+,
                    other => Err(ConversionError::new::<Self, _>(other)),
                }
            }
        }

        gen_std_conversion!($T);
        gen_std_conversion!(Option<$T>);
    }
}

gen_conversion!(bool; value::Inner::Boolean(x) => x);
gen_conversion!(i64; value::Inner::Int(x) => x);
gen_conversion!(u32; value::Inner::Date(x) => x);
gen_conversion!(u64; value::Inner::Time(x) => x);
gen_conversion!(f32; value::Inner::Float(x) => x);
gen_conversion!(f64; value::Inner::Double(x) => x);
gen_conversion!(Decimal; value::Inner::Decimal(x) => x);
gen_conversion!(Inet; value::Inner::Inet(x) => x);
gen_conversion!(String; value::Inner::String(x) => x);
gen_conversion!(UdtValue; value::Inner::Udt(x) => x);
gen_conversion!(Uuid; value::Inner::Uuid(x) => x);
gen_conversion!(Varint; value::Inner::Varint(x) => x);
gen_conversion!(Vec<u8>;
    value::Inner::Bytes(x) => x,
    value::Inner::Uuid(x) => x.value);

/// Counts the number of arguments
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

/// Generates `TryFromValue` and `TryFrom<Value>` implementations for tuples of fixed size,
/// denoted by the number of arguments.
macro_rules! gen_convert_value_tuple {
    ($($T:ident),+) => {

        // for 2-ary tuples expands to: `impl <A2, A1> TryFromValue for (A2, A1)`
        impl<$($T),+> TryFromValue for ($($T),+)
        where $($T: TryFromValue),+
        {
            fn try_from(value: Value) -> Result<Self, ConversionError> {
                match value.inner {
                    // if the size doesn't match, we just bail out in the `other` case
                    Some(value::Inner::Collection(c)) if c.elements.len() == count!($($T)+) => {
                        let mut i = c.elements.into_iter();
                        Ok((
                            $({ let x: $T = i.next().unwrap().try_into()?; x }),+
                        ))
                    }
                    other => Err(ConversionError::new::<Self, _>(other)),
                }
            }
        }

        // for 2-ary tuples expands to: `gen_std_conversion_generic!(<A2, A1> (A2, A1))`
        gen_std_conversion_generic!(<$($T),+> ($($T),+));
    }
}

/// Calls `gen_convert_value_tuple!` recursively to generate conversions for all tuples
/// starting at size 2 and ending at the size specified by the number of arguments.
macro_rules! gen_convert_value_all_tuples {
    ($first:ident) => {};
    ($first:ident, $($tail:ident),*) => {
        gen_convert_value_tuple!($first, $($tail),*);
        gen_convert_value_all_tuples!($($tail),*);
    }
}

// Generate conversions for all tuples up to size 16
gen_convert_value_all_tuples!(
    A16, A15, A14, A13, A12, A11, A10, A9, A8, A7, A6, A5, A4, A3, A2, A1
);

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
            other => Err(ConversionError::new::<Self, _>(other)),
        }
    }
}

/// Maps are passed as collections of key-value pairs, where items (0, 2, 4, ...) are keys,
/// and items (1, 3, 5, ...) are values. This means key-value pairs are not encoded as nested
/// collections. Hence, in order to receive a map, we must convert it to `Vec<KeyValue<K, V>>`
/// and *not* into `Vec<(K, V)>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue<K, V>(pub K, pub V);

impl<K, V> KeyValue<K, V> {
    pub fn into_tuple(self) -> (K, V) {
        (self.0, self.1)
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
            other => Err(ConversionError::new::<Self, _>(other)),
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
    fn convert_value_to_string() {
        let v = Value::string("foo".to_string());
        let s: String = v.try_into().unwrap();
        assert_eq!(s, "foo".to_string())
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
}
