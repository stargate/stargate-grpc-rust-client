mod from_value;
mod into_value;

tonic::include_proto!("stargate");

pub use from_value::*;
pub use into_value::*;

pub mod types {
    pub struct Boolean;
    pub struct Bytes;
    pub struct Date;
    pub struct Double;
    pub struct Float;
    pub struct Inet;
    pub struct Int;
    pub struct String;
    pub struct Time;
    pub struct Udt;
    pub struct Varint;

    pub struct List<T>(pub T);
    pub struct Map<K, V>(pub K, pub V);
}
