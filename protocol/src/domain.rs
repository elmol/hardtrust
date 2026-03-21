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

/// A signed capture produced by a device after executing an external command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capture {
    pub serial: String,
    pub address: String,
    pub timestamp: String,
    pub content_hash: String,
    pub files: Vec<CaptureFile>,
    pub signature: String,
}
