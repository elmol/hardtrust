/// Errors that can occur during protocol signing operations.
#[derive(Debug)]
pub enum ProtocolError {
    /// The reading contains an invalid Ethereum address.
    InvalidAddress(String),
    /// The reading contains an invalid or unparseable timestamp.
    InvalidTimestamp(String),
    /// The ECDSA signing operation failed.
    SigningFailed(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::InvalidAddress(s) => write!(f, "invalid address: {s}"),
            ProtocolError::InvalidTimestamp(s) => write!(f, "invalid timestamp: {s}"),
            ProtocolError::SigningFailed(s) => write!(f, "signing failed: {s}"),
        }
    }
}

impl std::error::Error for ProtocolError {}
