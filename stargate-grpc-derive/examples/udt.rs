use stargate_grpc::*;

#[derive(Debug, IntoValue, TryFromValue)]
struct Address {
    pub street: String,
    pub number: i64,
}

fn main() {
    let address = Address {
        street: "Zardzewiała".to_string(),
        number: 55,
    };
    let value = Value::from(address);
    println!("Got value: {:?}", value);

    let back_to_address = Address::try_from(value).unwrap();
    println!("Converted back to: {:?}", back_to_address);
}
