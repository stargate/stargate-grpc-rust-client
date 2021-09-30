mod from_value;
mod into_value;

tonic::include_proto!("stargate");

pub use from_value::*;
pub use into_value::*;

pub mod types {

    /// Must be implemented by all types except Any.
    pub trait ConcreteType {}

    pub struct Boolean;
    impl ConcreteType for Boolean {}
    pub struct Bytes;
    impl ConcreteType for Bytes {}
    pub struct Date;
    impl ConcreteType for Date {}
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
