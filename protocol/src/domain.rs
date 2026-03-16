use serde::{Deserialize, Serialize};

/// A signed data reading emitted by a device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reading {
    pub serial: String,
    pub address: String,
    pub temperature: f64,
    pub timestamp: String,
    pub signature: String,
}
