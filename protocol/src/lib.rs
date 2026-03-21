pub mod crypto;
pub mod domain;
pub mod error;

pub use crypto::{
    public_key_to_address, reading_prehash, sign, sign_reading, verify, verify_reading, Signable,
};
pub use domain::{Capture, CaptureFile, Reading};
pub use error::ProtocolError;

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;

    fn test_signing_key() -> k256::ecdsa::SigningKey {
        let key_bytes =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("valid hex");
        k256::ecdsa::SigningKey::from_slice(&key_bytes).expect("valid key")
    }

    fn test_reading() -> Reading {
        Reading {
            serial: "TEST-001".to_string(),
            address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            temperature: 42.0,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            signature: "0x".to_string(),
        }
    }

    fn signed_test_reading() -> (Reading, Address) {
        let key = test_signing_key();
        let mut reading = test_reading();
        reading.signature = sign_reading(&key, &reading).expect("valid reading");
        let address = public_key_to_address(key.verifying_key());
        (reading, address)
    }

    #[test]
    fn verify_reading_returns_true_for_valid_signature() {
        let (reading, address) = signed_test_reading();
        assert!(verify_reading(&reading, address));
    }

    #[test]
    fn verify_reading_returns_false_for_tampered_temperature() {
        let (mut reading, address) = signed_test_reading();
        reading.temperature = 99.0;
        assert!(!verify_reading(&reading, address));
    }

    #[test]
    fn verify_reading_returns_false_for_tampered_timestamp() {
        let (mut reading, address) = signed_test_reading();
        reading.timestamp = "2025-01-01T00:00:00Z".to_string();
        assert!(!verify_reading(&reading, address));
    }

    #[test]
    fn verify_reading_returns_false_for_tampered_serial() {
        let (mut reading, address) = signed_test_reading();
        reading.serial = "TAMPERED".to_string();
        assert!(!verify_reading(&reading, address));
    }

    #[test]
    fn verify_reading_returns_false_for_fake_signature() {
        let (mut reading, address) = signed_test_reading();
        reading.signature = "0xFAKESIG".to_string();
        assert!(!verify_reading(&reading, address));
    }

    #[test]
    fn verify_reading_returns_false_for_zero_address() {
        let (reading, _) = signed_test_reading();
        assert!(!verify_reading(&reading, Address::ZERO));
    }

    #[test]
    fn sign_reading_round_trip_recovers_signer_address() {
        use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
        use sha3::{Digest, Keccak256};

        let key = test_signing_key();
        let reading = test_reading();
        let sig_hex = sign_reading(&key, &reading).expect("valid reading");

        let expected_address = public_key_to_address(key.verifying_key());

        // Decode signature
        let sig_bytes = hex::decode(&sig_hex[2..]).expect("valid hex");
        assert_eq!(sig_bytes.len(), 65);
        let sig = Signature::from_slice(&sig_bytes[..64]).expect("valid sig");
        let rec_id = RecoveryId::from_byte(sig_bytes[64]).expect("valid rec id");

        // Reconstruct hash (same logic as sign_reading)
        let serial_hash: [u8; 32] = Keccak256::digest(reading.serial.as_bytes()).into();
        let addr_str = reading.address.trim_start_matches("0x");
        let address_bytes: [u8; 20] = hex::decode(addr_str).unwrap().try_into().unwrap();
        let temp_scaled = (reading.temperature * 1000.0) as i64;
        let ts = chrono::DateTime::parse_from_rfc3339(&reading.timestamp)
            .unwrap()
            .timestamp() as u64;
        let mut preimage = Vec::with_capacity(68);
        preimage.extend_from_slice(&serial_hash);
        preimage.extend_from_slice(&address_bytes);
        preimage.extend_from_slice(&temp_scaled.to_be_bytes());
        preimage.extend_from_slice(&ts.to_be_bytes());
        let hash = Keccak256::digest(&preimage);

        let recovered_key =
            VerifyingKey::recover_from_prehash(hash.as_ref(), &sig, rec_id).expect("recover");
        let recovered_address = public_key_to_address(&recovered_key);

        assert_eq!(recovered_address, expected_address);
    }

    #[test]
    fn sign_reading_is_deterministic() {
        let key = test_signing_key();
        let reading = test_reading();
        let sig1 = sign_reading(&key, &reading).expect("valid reading");
        let sig2 = sign_reading(&key, &reading).expect("valid reading");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn public_key_to_address_matches_anvil_account_0() {
        use k256::ecdsa::SigningKey;
        let key_bytes =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("valid hex");
        let signing_key = SigningKey::from_slice(&key_bytes).expect("valid key");
        let verifying_key = signing_key.verifying_key();
        let address = public_key_to_address(verifying_key);
        let expected: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .expect("valid address");
        assert_eq!(address, expected);
    }

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
    fn reading_prehash_returns_expected_hash() {
        let reading = test_reading();
        let hash = reading_prehash(&reading).expect("valid reading");
        let expected = {
            use sha3::{Digest, Keccak256};
            let serial_hash: [u8; 32] = Keccak256::digest(reading.serial.as_bytes()).into();
            let addr_str = reading.address.trim_start_matches("0x");
            let address_bytes: [u8; 20] = hex::decode(addr_str).unwrap().try_into().unwrap();
            let temp_scaled = (reading.temperature * 1000.0) as i64;
            let ts = chrono::DateTime::parse_from_rfc3339(&reading.timestamp)
                .unwrap()
                .timestamp() as u64;
            let mut preimage = Vec::with_capacity(68);
            preimage.extend_from_slice(&serial_hash);
            preimage.extend_from_slice(&address_bytes);
            preimage.extend_from_slice(&temp_scaled.to_be_bytes());
            preimage.extend_from_slice(&ts.to_be_bytes());
            let h: [u8; 32] = Keccak256::digest(&preimage).into();
            h
        };
        assert_eq!(hash, expected);
    }

    #[test]
    fn reading_prehash_returns_none_for_invalid_address() {
        let mut reading = test_reading();
        reading.address = "0xZZZZ".to_string();
        assert!(reading_prehash(&reading).is_none());
    }

    #[test]
    fn reading_prehash_returns_none_for_invalid_timestamp() {
        let mut reading = test_reading();
        reading.timestamp = "not-a-timestamp".to_string();
        assert!(reading_prehash(&reading).is_none());
    }

    #[test]
    fn reading_missing_field_fails_to_deserialize() {
        let json = r#"{"serial":"X","address":"Y","temperature":1.0,"timestamp":"Z"}"#;
        let result = serde_json::from_str::<Reading>(json);
        assert!(result.is_err());
    }

    #[test]
    fn sign_reading_returns_err_for_invalid_address() {
        let key = test_signing_key();
        let mut reading = test_reading();
        reading.address = "0xZZZZ".to_string();
        let result = sign_reading(&key, &reading);
        assert!(matches!(result, Err(ProtocolError::InvalidAddress(_))));
    }

    #[test]
    fn sign_reading_returns_err_for_invalid_timestamp() {
        let key = test_signing_key();
        let mut reading = test_reading();
        reading.timestamp = "not-a-timestamp".to_string();
        let result = sign_reading(&key, &reading);
        assert!(matches!(result, Err(ProtocolError::InvalidTimestamp(_))));
    }

    // --- Capture tests ---

    fn test_capture() -> Capture {
        Capture {
            serial: "TERRA-001".to_string(),
            address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            content_hash: "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
            files: vec![CaptureFile {
                name: "image.png".to_string(),
                hash: "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                    .to_string(),
                size: 1024,
            }],
            signature: "0x".to_string(),
        }
    }

    fn signed_test_capture() -> (Capture, Address) {
        let key = test_signing_key();
        let mut capture = test_capture();
        capture.signature = sign(&key, &capture).expect("valid capture");
        let address = public_key_to_address(key.verifying_key());
        (capture, address)
    }

    #[test]
    fn sign_verify_capture_round_trip() {
        let (capture, address) = signed_test_capture();
        assert!(verify(&capture, address));
    }

    #[test]
    fn capture_tampered_content_hash_fails_verify() {
        let (mut capture, address) = signed_test_capture();
        capture.content_hash =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert!(!verify(&capture, address));
    }

    #[test]
    fn capture_tampered_timestamp_fails_verify() {
        let (mut capture, address) = signed_test_capture();
        capture.timestamp = "2025-06-01T00:00:00Z".to_string();
        assert!(!verify(&capture, address));
    }

    #[test]
    fn capture_serializes_deserializes_via_serde() {
        let capture = test_capture();
        let json = serde_json::to_string(&capture).expect("serialize");
        let deserialized: Capture = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(capture, deserialized);
    }

    #[test]
    fn capture_prehash_returns_none_for_invalid_address() {
        let mut capture = test_capture();
        capture.address = "0xZZZZ".to_string();
        assert!(capture.prehash().is_none());
    }

    #[test]
    fn capture_prehash_returns_none_for_invalid_content_hash() {
        let mut capture = test_capture();
        capture.content_hash = "not-sha256-prefixed".to_string();
        assert!(capture.prehash().is_none());
    }

    #[test]
    fn capture_prehash_returns_none_for_short_content_hash() {
        let mut capture = test_capture();
        capture.content_hash = "sha256:abcd".to_string();
        assert!(capture.prehash().is_none());
    }

    #[test]
    fn sign_capture_returns_err_for_invalid_payload() {
        let key = test_signing_key();
        let mut capture = test_capture();
        capture.address = "0xZZZZ".to_string();
        let result = sign(&key, &capture);
        assert!(matches!(result, Err(ProtocolError::InvalidPayload)));
    }
}
