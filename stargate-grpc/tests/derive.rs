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
    fn default_permissions() -> i64 {
        0o660
    }

    #[derive(TryFromValue)]
    struct File {
        path: String,
        #[stargate(default)]
        symlink: bool,
        #[stargate(default = "default_permissions")]
        permissions: i64,
    }
    let udt_value = Value::udt(vec![("path", Value::string("file"))]);
    let file: File = udt_value.try_into().unwrap();
    assert_eq!(file.path, "file".to_string());
    assert_eq!(file.symlink, false);
    assert_eq!(file.permissions, 0o660);
}
