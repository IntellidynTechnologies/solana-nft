use solana_client::client_error::ClientError;

#[derive(Debug)]
pub enum CustomError {
    ConfigReadError,
    ConfigParseError,
    InvalidConfig,
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