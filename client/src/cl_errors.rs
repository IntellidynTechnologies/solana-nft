use solana_client::client_error::ClientError;

#[derive(Debug, PartialEq)]
pub enum CustomError {
    ConfigReadError,
    ConfigParseError,
    InvalidConfig,
    InvalidInput,
    SerializationError,
    ClientError,
    KeyDerivationError,
    Custom(String)
}

impl From<ClientError> for CustomError {
    fn from(client_error: ClientError) -> Self {
        CustomError::Custom(client_error.to_string())
    }
}