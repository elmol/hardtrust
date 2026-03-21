/// Errors that can occur during protocol signing operations.
#[derive(Debug, PartialEq)]
pub enum ProtocolError {
    /// The reading contains an invalid Ethereum address.
    InvalidAddress(String),
    /// The reading contains an invalid or unparseable timestamp.
    InvalidTimestamp(String),
    /// The payload data is invalid for signing (bad fields that prevent prehash).
    InvalidPayload,
    /// The ECDSA signing operation failed.
    SigningFailed(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::InvalidAddress(s) => write!(f, "invalid address: {s}"),
            ProtocolError::InvalidTimestamp(s) => write!(f, "invalid timestamp: {s}"),
            ProtocolError::InvalidPayload => write!(f, "invalid payload: prehash failed"),
            ProtocolError::SigningFailed(s) => write!(f, "signing failed: {s}"),
        }
    }
}

impl std::error::Error for ProtocolError {}
