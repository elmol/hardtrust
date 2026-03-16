pub mod dev_config;

use alloy_primitives::{keccak256, Address, Signature as AlloySignature, B256};
use k256::ecdsa::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

/// Errors that can occur during core signing operations.
#[derive(Debug)]
pub enum CoreError {
    /// The reading contains an invalid Ethereum address.
    InvalidAddress(String),
    /// The reading contains an invalid or unparseable timestamp.
    InvalidTimestamp(String),
    /// The ECDSA signing operation failed.
    SigningFailed(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::InvalidAddress(s) => write!(f, "invalid address: {s}"),
            CoreError::InvalidTimestamp(s) => write!(f, "invalid timestamp: {s}"),
            CoreError::SigningFailed(s) => write!(f, "signing failed: {s}"),
        }
    }
}

impl std::error::Error for CoreError {}

/// Derives an Ethereum address from a secp256k1 public key.
///
/// Encodes the key as uncompressed bytes, strips the 0x04 prefix,
/// applies keccak256 to the 64-byte body, and returns the last 20 bytes.
pub fn public_key_to_address(pk: &VerifyingKey) -> Address {
    let encoded = pk.to_encoded_point(false);
    let bytes = encoded.as_bytes();
    // bytes[0] is 0x04 prefix; skip it
    let hash = keccak256(&bytes[1..]);
    Address::from_slice(&hash[12..])
}

/// Build the canonical keccak256 hash of a Reading's signing payload.
///
/// The 68-byte preimage is: `serial_hash(32) || address_bytes(20) || temperature_scaled(8) || timestamp_u64(8)`.
/// Returns `None` if the reading has invalid address hex or timestamp format.
pub fn reading_prehash(reading: &Reading) -> Option<[u8; 32]> {
    let serial_hash: [u8; 32] = Keccak256::digest(reading.serial.as_bytes()).into();

    let addr_str = reading.address.trim_start_matches("0x");
    let address_bytes: [u8; 20] = hex::decode(addr_str).ok()?.try_into().ok()?;

    let temp_scaled = (reading.temperature * 1000.0) as i64;

    let ts = chrono::DateTime::parse_from_rfc3339(&reading.timestamp)
        .ok()?
        .timestamp() as u64;

    let mut preimage = Vec::with_capacity(68);
    preimage.extend_from_slice(&serial_hash);
    preimage.extend_from_slice(&address_bytes);
    preimage.extend_from_slice(&temp_scaled.to_be_bytes());
    preimage.extend_from_slice(&ts.to_be_bytes());
    debug_assert_eq!(preimage.len(), 68);

    Some(Keccak256::digest(&preimage).into())
}

/// Signs a `Reading` and returns a 65-byte EVM-format hex signature.
///
/// The payload is `keccak256(serial_hash || address_bytes || temperature_scaled || timestamp_u64)`
/// where the 68-byte preimage is constructed as specified in S1b.2-V1.
/// Returns `"0x" + hex(r || s || v)` where v ∈ {0, 1}.
/// Returns `Err` if the reading contains an invalid address, timestamp, or signing fails.
pub fn sign_reading(key: &SigningKey, reading: &Reading) -> Result<String, CoreError> {
    // Validate address
    let addr_str = reading.address.trim_start_matches("0x");
    if hex::decode(addr_str)
        .ok()
        .and_then(|b| <[u8; 20]>::try_from(b).ok())
        .is_none()
    {
        return Err(CoreError::InvalidAddress(reading.address.clone()));
    }

    // Validate timestamp
    if chrono::DateTime::parse_from_rfc3339(&reading.timestamp).is_err() {
        return Err(CoreError::InvalidTimestamp(reading.timestamp.clone()));
    }

    let hash = reading_prehash(reading).expect("already validated above");

    let (sig, recovery_id) = key
        .sign_prehash_recoverable(hash.as_ref())
        .map_err(|e| CoreError::SigningFailed(e.to_string()))?;

    let r = sig.r().to_bytes();
    let s = sig.s().to_bytes();
    let v = recovery_id.to_byte();

    let mut bytes = Vec::with_capacity(65);
    bytes.extend_from_slice(&r);
    bytes.extend_from_slice(&s);
    bytes.push(v);

    Ok(format!("0x{}", hex::encode(&bytes)))
}

/// Verifies a `Reading`'s ECDSA signature against an expected on-chain address.
///
/// Reconstructs the canonical payload hash (identical to `sign_reading`),
/// recovers the signer address from the signature, and returns `true` only if
/// recovery succeeds, the recovered address matches `on_chain_address`, and
/// `on_chain_address` is not the zero address.
///
/// Returns `false` — never panics — for any malformed or unrecoverable signature.
/// See ADR-0007 for why no Ethereum personal sign prefix is used.
pub fn verify_reading(reading: &Reading, on_chain_address: Address) -> bool {
    if on_chain_address == Address::ZERO {
        return false;
    }

    let hash = match reading_prehash(reading) {
        Some(h) => h,
        None => return false,
    };
    let prehash = B256::from(hash);

    // Parse signature
    let sig_hex = reading.signature.trim_start_matches("0x");
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let alloy_sig = match AlloySignature::from_raw(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Recover and compare
    match alloy_sig.recover_address_from_prehash(&prehash) {
        Ok(recovered) => recovered == on_chain_address,
        Err(_) => false,
    }
}

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
        // Precomputed keccak256 of the canonical 68-byte preimage for test_reading()
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
        assert!(matches!(result, Err(CoreError::InvalidAddress(_))));
    }

    #[test]
    fn sign_reading_returns_err_for_invalid_timestamp() {
        let key = test_signing_key();
        let mut reading = test_reading();
        reading.timestamp = "not-a-timestamp".to_string();
        let result = sign_reading(&key, &reading);
        assert!(matches!(result, Err(CoreError::InvalidTimestamp(_))));
    }
}
