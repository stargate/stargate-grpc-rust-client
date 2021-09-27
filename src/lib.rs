use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use itertools::*;

tonic::include_proto!("stargate");

#[derive(Clone, Debug)]
pub struct DataTypeError {
    cql_value: String,
    rust_type_name: &'static str,
}

impl DataTypeError {
    fn new<T, V: Debug>(cql_value: V) -> DataTypeError {
        DataTypeError {
            cql_value: format!("{:?}", cql_value),
            rust_type_name: std::any::type_name::<T>(),
        }
    }
}

impl Display for DataTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot convert value {} to {}",
            self.cql_value, self.rust_type_name
        )
    }
}

impl Error for DataTypeError {}

impl TryFrom<Value> for bool {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Boolean(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Int(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for u32 {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Date(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for u64 {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Time(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for f32 {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Float(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Double(v)) => Ok(v),
            Some(value::Inner::Float(v)) => Ok(v as f64),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Decimal {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Decimal(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Bytes(v)) => Ok(v),
            Some(value::Inner::Uuid(v)) => Ok(v.value),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Inet {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Inet(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Uuid {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Uuid(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::String(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl<T> TryFrom<Value> for Vec<T>
where
    T: TryFrom<Value, Error = DataTypeError>,
{
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Collection(v)) => {
                Ok(v.elements.into_iter().map(|e| e.try_into()).try_collect()?)
            }
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for UdtValue {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Udt(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Varint {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            Some(value::Inner::Varint(v)) => Ok(v),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Option<bool> {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            None => Ok(None),
            Some(value::Inner::Null(_)) => Ok(None),
            Some(value::Inner::Unset(_)) => Ok(None),
            Some(value::Inner::Boolean(v)) => Ok(Some(v)),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Option<i64> {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            None => Ok(None),
            Some(value::Inner::Null(_)) => Ok(None),
            Some(value::Inner::Unset(_)) => Ok(None),
            Some(value::Inner::Int(v)) => Ok(Some(v)),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}

impl TryFrom<Value> for Option<String> {
    type Error = DataTypeError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.inner {
            None => Ok(None),
            Some(value::Inner::String(v)) => Ok(Some(v)),
            other => Err(DataTypeError::new::<Self, _>(other)),
        }
    }
}
