//! Automatic conversions from standard Rust types to `Value`

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use itertools::Itertools;

use crate::types::{List, Map};
use crate::*;

/// Selects the default Cassandra gRPC value type associated with a Rust type.
/// The default type is used when a Rust value `x` is converted to `Value` by calling
/// `x.into()` or `Value::from(x)`.
///
/// In order to convert a Rust value to a non-default Cassandra type,
/// use [`Value::of_type()`].
///
/// # Default type mapping
///
/// | Rust type        | gRPC type       |
/// |------------------|-----------------|
/// | `i8`             | `Int`           |
/// | `i16`            | `Int`           |
/// | `i32`            | `Int`           |
/// | `i64`            | `Int`           |
/// | `f32`            | `Float`         |
/// | `f64`            | `Double`        |
/// | `bool`           | `Boolean`       |
/// | `String`         | `String`        |
/// | `Vec<u8>`        | `Bytes`         |
/// | `Vec<T>`         | `List`          |
/// | `Vec<KeyValue>`  | `Map`           |
/// | `HashMap<K, V>`  | `Map`           |
/// | `BTreeMap<K, V>` | `Map            |
/// | `(T1, T2, ...)`  | `List`          |
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
    /// gRPC type, must be set to one of the types defined in the `types` module.
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
    /// the [`types`] module. This is useful when a non-default conversion is needed.
    ///
    /// # Example
    /// ```
    /// use stargate_grpc::Value;
    /// use stargate_grpc::types::*;
    ///
    /// // by default Vec<u8> get converted to Bytes, but here we want to convert them to Inet
    /// let addr1 = vec![127, 0, 0, 1];
    /// let addr2 = vec![127, 0, 0, 2];
    /// let expected = Value::list(vec![Value::inet(addr1.clone()), Value::inet(addr2.clone())]);
    /// let addresses = Value::of_type(List(Inet), vec![addr1, addr2]);
    /// assert_eq!(addresses, expected);
    /// ```
    pub fn of_type<R: IntoValue<C>, C>(_cassandra_type: C, value: R) -> Value {
        value.into_value()
    }

    /// Creates a Cassandra Null value.
    pub fn null() -> Value {
        Value {
            inner: Some(value::Inner::Null(value::Null {})),
        }
    }

    /// Unset value. Unset query parameter values are ignored by the server.
    ///
    /// Use this value if you need to bind a parameter in an insert statement,
    /// but you don't want to change the target value stored in the database.
    /// To be used only for bind values in queries.
    pub fn unset() -> Value {
        Value {
            inner: Some(value::Inner::Unset(value::Unset {})),
        }
    }

    pub fn boolean(value: bool) -> Value {
        Value {
            inner: Some(value::Inner::Boolean(value)),
        }
    }

    pub fn int(value: i64) -> Value {
        Value {
            inner: Some(value::Inner::Int(value)),
        }
    }

    pub fn float(value: f32) -> Value {
        Value {
            inner: Some(value::Inner::Float(value)),
        }
    }

    pub fn double(value: f64) -> Value {
        Value {
            inner: Some(value::Inner::Double(value)),
        }
    }

    pub fn date(value: u32) -> Value {
        Value {
            inner: Some(value::Inner::Date(value)),
        }
    }

    pub fn time(value: u64) -> Value {
        Value {
            inner: Some(value::Inner::Time(value)),
        }
    }

    pub fn uuid(value: Vec<u8>) -> Value {
        Value {
            inner: Some(value::Inner::Uuid(Uuid { value })),
        }
    }

    pub fn inet(value: Vec<u8>) -> Value {
        Value {
            inner: Some(value::Inner::Inet(Inet { value })),
        }
    }

    pub fn bytes(value: Vec<u8>) -> Value {
        Value {
            inner: Some(value::Inner::Bytes(value)),
        }
    }

    pub fn varint(value: Vec<u8>) -> Value {
        Value {
            inner: Some(value::Inner::Varint(Varint { value })),
        }
    }

    pub fn decimal(scale: u32, value: Vec<u8>) -> Value {
        Value {
            inner: Some(value::Inner::Decimal(Decimal { scale, value })),
        }
    }

    pub fn string<S: ToString>(value: S) -> Value {
        Value {
            inner: Some(value::Inner::String(value.to_string())),
        }
    }

    /// Converts an iterable collection to a `Value` representing a list.
    /// Items are converted to `Value` if needed.
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
    pub fn list<I, T>(elements: I) -> Value
    where
        I: IntoIterator<Item = T>,
        T: Into<Value>,
    {
        let elements = elements.into_iter().map(|e| e.into()).collect_vec();
        Value {
            inner: Some(value::Inner::Collection(Collection { elements })),
        }
    }

    /// Converts a collection of key-value pairs to a `Value` representing a map.
    /// Keys and values are converted to `Value` as needed.
    ///
    /// # Type Parameters
    /// - `K`: type of the keys
    /// - `V`: type of the values
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
    pub fn map<I, K, V>(elements: I) -> Value
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Value>,
        V: Into<Value>,
    {
        let iter = elements.into_iter();
        let (size_hint_lower, size_hint_upper) = iter.size_hint();
        let mut collection = Vec::with_capacity(size_hint_upper.unwrap_or(size_hint_lower) * 2);
        for (k, v) in iter {
            collection.push(k.into());
            collection.push(v.into());
        }
        Value {
            inner: Some(value::Inner::Collection(Collection {
                elements: collection,
            })),
        }
    }

    pub fn udt<K, V>(fields: HashMap<K, V>) -> Value
    where
        K: ToString,
        V: Into<Value>,
    {
        let fields = fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.into()))
            .collect();
        Value {
            inner: Some(value::Inner::Udt(UdtValue { fields })),
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
gen_conversion!(Vec<u8> => types::Inet; x => Value::inet(x));
gen_conversion!(Vec<u8> => types::Varint; x => Value::varint(x));

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
{
    fn into_value(self) -> Value {
        match self {
            None => Value::null(),
            Some(v) => v.into_value(),
        }
    }
}

impl<R, C> IntoValue<List<C>> for Vec<R>
where
    R: IntoValue<C>,
{
    fn into_value(self) -> Value {
        let elements = self.into_iter().map(|e| e.into_value()).collect_vec();
        Value::list(elements)
    }
}

impl<RK, RV, CK, CV> IntoValue<Map<CK, CV>> for Vec<(RK, RV)>
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

impl<RK, RV, CK, CV> IntoValue<Map<CK, CV>> for Vec<KeyValue<RK, RV>>
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

impl<RK, RV, CK, CV> IntoValue<Map<CK, CV>> for BTreeMap<RK, RV>
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

impl<RK, RV, CK, CV> IntoValue<Map<CK, CV>> for HashMap<RK, RV>
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

    use crate::types::{Date, Int, List, Map, Time};
    use crate::*;

    #[test]
    fn convert_i64_into_value() {
        let v: Value = 100.into();
        assert_eq!(v, Value::int(100));
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
    fn convert_tuple_to_default_value() {
        let tuple = (1, "foo");
        let v = Value::from(tuple);
        assert_eq!(v, Value::list(vec![Value::int(1), Value::string("foo")]))
    }

    #[test]
    fn convert_tuple_to_typed_value() {
        let tuple = (1, 100);
        let v = Value::of_type((Int, Time), tuple);
        assert_eq!(v, Value::list(vec![Value::int(1), Value::time(100)]))
    }

    #[test]
    fn convert_large_tuple_to_value() {
        let tuple = (1, 2, 3, 4, 5, "foo");
        let v = Value::from(tuple);
        match v.inner {
            Some(value::Inner::Collection(value)) if value.elements.len() == 6 => {}
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
            Some(value::Inner::Udt(value)) if value.fields.len() == 2 => {}
            inner => assert!(false, "Unexpected udt inner value {:?}", inner),
        }
    }
}
