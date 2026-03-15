pub mod dev_config;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_reading() -> Reading {
        Reading {
            serial: "TEST-001".to_string(),
            address: "0xabcd".to_string(),
            temperature: 42.0,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            signature: "0xFAKESIG".to_string(),
        }
    }

    #[test]
    fn reading_serializes_to_json_with_all_fields() {
        let json = serde_json::to_string(&sample_reading()).expect("serialize");
        assert!(json.contains("serial"));
        assert!(json.contains("address"));
        assert!(json.contains("temperature"));
        assert!(json.contains("timestamp"));
        assert!(json.contains("signature"));
    }

    #[test]
    fn reading_round_trips_through_serde() {
        let reading = sample_reading();
        let json = serde_json::to_string(&reading).expect("serialize");
        let deserialized: Reading = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(reading, deserialized);
    }

    #[test]
    fn reading_missing_field_fails_to_deserialize() {
        let json = r#"{"serial":"X","address":"Y","temperature":1.0,"timestamp":"Z"}"#;
        let result = serde_json::from_str::<Reading>(json);
        assert!(result.is_err());
    }
}
