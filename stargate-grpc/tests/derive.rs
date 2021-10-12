//! Integration tests for stargate-grpc-derive

use stargate_grpc::error::{ConversionError, ConversionErrorKind};
use stargate_grpc::*;

#[test]
fn convert_struct_to_udt_value() {
    #[derive(IntoValue)]
    struct Address {
        street: &'static str,
        number: i64,
    }
    let addr = Address {
        street: "foo",
        number: 123,
    };
    let value = Value::from(addr);
    match value.inner {
        Some(stargate_grpc::proto::value::Inner::Udt(value)) => {
            assert_eq!(value.fields.get("street"), Some(&Value::string("foo")));
            assert_eq!(value.fields.get("number"), Some(&Value::int(123)));
        }
        inner => {
            assert!(false, "Unexpected udt inner value {:?}", inner)
        }
    }
}

#[test]
fn convert_struct_to_udt_value_with_typed_fields() {
    #[derive(IntoValue)]
    struct Date {
        #[stargate(grpc_type = "types::Date")]
        days: u32,
    }
    let days = Date { days: 34835 };
    let value = Value::from(days);
    match value.inner {
        Some(stargate_grpc::proto::value::Inner::Udt(value)) => {
            assert_eq!(value.fields.get("days"), Some(&Value::date(34835)));
        }
        inner => {
            assert!(false, "Unexpected udt inner value {:?}", inner)
        }
    }
}

#[test]
fn convert_struct_to_value_skip_fields() {
    #[derive(IntoValue)]
    struct Address {
        street: &'static str,
        #[stargate(skip)] // exclude this field from writing into `UdtValue`
        #[allow(unused)]
        number: i64,
    }
    let addr = Address {
        street: "foo",
        number: 123,
    };
    let value = Value::from(addr);
    match value.inner {
        Some(stargate_grpc::proto::value::Inner::Udt(value)) => {
            assert_eq!(value.fields.get("street"), Some(&Value::string("foo")));
            assert_eq!(value.fields.get("number"), None);
        }
        inner => {
            assert!(false, "Unexpected udt inner value {:?}", inner)
        }
    }
}

#[test]
fn rename_fields() {
    #[derive(Eq, PartialEq, IntoValue, TryFromValue)]
    struct Address {
        #[stargate(name = "st")]
        street: String,
        number: i64,
    }
    let addr = Address {
        street: "foo".to_string(),
        number: 123,
    };
    let value = Value::from(addr);
    match &value.inner {
        Some(stargate_grpc::proto::value::Inner::Udt(value)) => {
            assert_eq!(value.fields.get("st"), Some(&Value::string("foo")));
            assert_eq!(value.fields.get("number"), Some(&Value::int(123)));
        }
        inner => {
            assert!(false, "Unexpected udt inner value {:?}", inner)
        }
    }
    // convert back
    let addr: Address = value.try_into().unwrap();
    assert_eq!(addr.street, "foo".to_string());
    assert_eq!(addr.number, 123);
}

#[test]
fn convert_udt_value_to_struct() {
    #[derive(TryFromValue)]
    struct Address {
        street: String,
        number: i64,
    }
    let udt_value = Value::udt(vec![
        ("street", Value::string("foo")),
        ("number", Value::int(123)),
    ]);
    let address: Address = udt_value.try_into().unwrap();
    assert_eq!(address.street, "foo".to_string());
    assert_eq!(address.number, 123);
}

#[test]
fn convert_udt_value_to_struct_with_default() {
    fn default_path() -> String {
        "file.cfg".to_string()
    }

    #[derive(TryFromValue)]
    struct ConfigFile {
        #[stargate(default = "default_path()")]
        path: String,
        #[stargate(default)]
        open_on_startup: bool,
        #[stargate(default)]
        write_lock: bool,
    }
    let udt_value = Value::udt(vec![
        ("path", Value::null()),
        ("write_lock", Value::boolean(true)),
    ]);
    let file: ConfigFile = udt_value.try_into().unwrap();
    assert_eq!(file.path, default_path());
    assert_eq!(file.open_on_startup, false);
    assert_eq!(file.write_lock, true);
}

#[test]
fn convert_udt_value_to_struct_missing_fields() {
    #[derive(TryFromValue)]
    #[allow(unused)]
    struct Address {
        street: String,
        number: i64,
    }
    let udt_value = Value::udt(vec![("number", Value::int(123))]);
    let result: Result<Address, ConversionError> = udt_value.try_into();
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().kind,
        ConversionErrorKind::FieldNotFound("street")
    )
}

#[test]
fn bind_struct_in_query() {
    #[derive(IntoValues)]
    struct User {
        id: i64,
        login: &'static str,
    }
    let user = User {
        id: 1,
        login: "user",
    };
    let query = QueryBuilder::new()
        .query("INSERT INTO users(id, login) VALUES (:id, :login)")
        .bind(user)
        .build();

    use prost::Message;
    let values: proto::Values =
        proto::Values::decode(query.values.unwrap().data.unwrap().value.as_slice()).unwrap();
    assert_eq!(
        values.value_names,
        vec!["id".to_string(), "login".to_string()]
    );
    assert_eq!(values.values, vec![Value::int(1), Value::string("user")]);
}
