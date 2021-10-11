//! Enhances the automatically generated gRPC Stargate client with token-based authentication.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use tonic::codegen::InterceptedService;
use tonic::metadata::AsciiMetadataValue;
use tonic::service::Interceptor;
use tonic::transport::ClientTlsConfig;
use tonic::{Request, Status};

use crate::proto::stargate_client;

/// Error returned on an attempt to create an [`AuthToken`] from an invalid string.
#[derive(Clone, Debug)]
pub struct InvalidAuthToken(String);

impl Display for InvalidAuthToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid authentication token value format. Must be a valid HTTP header value string."
        )
    }
}

impl Error for InvalidAuthToken {}

/// Stores a token for authenticating to Stargate.
///
/// You can obtain the token by sending a POST request with a username and password
/// to "/v1/auth" on port 8081 of Stargate.
///
/// <pre>
/// curl -L -X POST 'http://127.0.0.2:8081/v1/auth' \
///      -H 'Content-Type: application/json' \
///      --data-raw '{
///          "username": "cassandra",
///          "password": "cassandra"
///      }'
///
/// {"authToken":"25b538f6-3092-4fd1-8dd4-e73408f2bd60"}
/// </pre>
///
/// # Example
/// ```rust
/// use std::str::FromStr;
/// use stargate_grpc::client::AuthToken;
///
/// let token = AuthToken::from_str("4fa77b65-c93b-4711-8cd3-62bfd9c5d411").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AuthToken(AsciiMetadataValue);

impl FromStr for AuthToken {
    type Err = InvalidAuthToken;

    /// Creates a new authentication token from a String.
    /// This will fail if the string is not a valid UUID.
    fn from_str(s: &str) -> Result<AuthToken, InvalidAuthToken> {
        let ascii_value =
            AsciiMetadataValue::from_str(s).map_err(|_| InvalidAuthToken(s.to_string()))?;
        Ok(AuthToken(ascii_value))
    }
}

/// Allows to use `AuthToken` as a Tonic request interceptor that
/// attaches its token value to request header "x-cassandra-token".
impl Interceptor for AuthToken {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let mut request = request;
        request
            .metadata_mut()
            .insert("x-cassandra-token", self.0.clone());
        Ok(request)
    }
}

/// Type alias for the most commonly used `StargateClient` type
/// with support for authentication.
pub type StargateClient =
    stargate_client::StargateClient<InterceptedService<tonic::transport::Channel, AuthToken>>;

impl StargateClient {
    /// Creates a new `StargateClient` wrapping given channel, attaching the authentication
    /// token to each request.
    pub fn with_auth(channel: tonic::transport::Channel, token: AuthToken) -> Self {
        stargate_client::StargateClient::with_interceptor(channel, token)
    }
}

/// Returns the default TLS config with root certificates imported from the OS.
pub fn default_tls_config() -> std::io::Result<ClientTlsConfig> {
    let mut rustls_config = tokio_rustls::rustls::ClientConfig::new();
    rustls_config.alpn_protocols.push(b"h2".to_vec());
    rustls_config.root_store = match rustls_native_certs::load_native_certs() {
        Ok(root_store) => root_store,
        Err((Some(root_store), _)) => root_store,
        Err((None, e)) => return Err(e),
    };
    Ok(ClientTlsConfig::default().rustls_client_config(rustls_config))
}