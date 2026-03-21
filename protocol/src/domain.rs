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

/// A file entry in a capture manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureFile {
    pub name: String,
    pub hash: String,
    pub size: u64,
}

/// Environment metadata captured alongside a capture for attestation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureEnvironment {
    pub script_hash: String,
    pub binary_hash: String,
    pub hw_serial: String,
    pub camera_info: String,
}

/// A signed capture produced by a device after executing an external command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capture {
    pub serial: String,
    pub address: String,
    pub timestamp: String,
    pub content_hash: String,
    pub files: Vec<CaptureFile>,
    pub environment: CaptureEnvironment,
    pub signature: String,
}
