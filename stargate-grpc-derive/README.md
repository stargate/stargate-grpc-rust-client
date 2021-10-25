# Derive macros for `stargate-grpc`

This crate provides the following derive macros:

 - `IntoValue` – enables converting a Rust struct to a `Value` of a user-defined CQL type; 
    use this when you want to bind a single UDT field in a query
 - `TryFromValue` – enables converting a `Value` representing a user-defined CQL type to a Rust struct; 
    use this to read a single UDT column value from a row
 - `IntoValues` – enables converting a Rust struct to many arguments of a query at once;  
    use this if you want to bind many fields of a single Rust struct in a single call to `bind` 
 - `TryFromRow` – enables converting a `Row` received in a result set to a Rust value

## Example
```rust
use stargate_grpc::Value;
use stargate_grpc_derive::{IntoValue, TryFromValue};

#[derive(IntoValue, TryFromValue)]
struct User {
    id: i64,
    login: String
}

let user = User { id: 1, login: "user".to_string() };

// Convert User to Value:
let value = Value::from(user);
assert_eq!(value, Value::udt(vec![("id", Value::bigint(1)), ("login", Value::string("user"))]));

// Now convert it back to User:
let user: User = value.try_into().unwrap();
assert_eq!(user.id, 1);
assert_eq!(user.login, "user".to_string());

```

See [crate documentation](https://docs.rs/stargate-grpc-derive) for more examples. 

