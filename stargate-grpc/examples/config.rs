//! Command-line configuration shared by all examples.
//! This is also an executable demo. It prints the received configuration to stdout.

use clap::Clap;
use stargate_grpc::AuthToken;

#[derive(Clap, Debug)]
#[clap(name = "Stargate Rust gRPC client demo program")]
pub struct Config {
    /// Keyspace name to use; on Astra it must match the keyspace of the database
    #[clap(short('k'), long, default_value = "stargate_examples")]
    pub keyspace: String,

    /// Authentication token.
    #[clap(short('t'), long, env("SG_TOKEN"), parse(try_from_str))]
    pub token: AuthToken,

    /// Enable TLS.
    #[clap(long)]
    pub tls: bool,

    /// Stargate coordinator URL
    #[clap(default_value = "http://127.0.0.2:8090")]
    pub url: String,
}

impl Config {
    pub fn from_args() -> Config {
        Config::parse()
    }
}

#[allow(unused)]
fn main() {
    println!("{:?}", Config::from_args());
}
