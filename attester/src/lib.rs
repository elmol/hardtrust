use alloy::hex;
use alloy::primitives::{keccak256, Address, FixedBytes, Signature as AlloySignature};
use hardtrust_core::{verify_reading, Reading};

/// Data required to register a device on-chain.
pub struct RegistrationData {
    /// keccak256 of the device serial number bytes.
    pub serial_hash: FixedBytes<32>,
}

/// Result of a device reading verification.
pub enum VerificationResult {
    /// The reading signature is valid and the signer matches the on-chain address.
    Verified,
    /// The reading could not be verified, with the specific reason.
    Unverified(UnverifiedReason),
}

/// Reason a reading verification failed.
pub enum UnverifiedReason {
    /// The signature field is not a parseable ECDSA signature.
    SignatureInvalid,
    /// The device address resolved to zero — not registered on-chain.
    DeviceNotRegistered,
    /// The signature is valid but the recovered address does not match the on-chain address.
    SignerMismatch,
}

/// Compute the registration data for a device serial number.
///
/// Pure: no blockchain interaction, no signing, no I/O.
pub fn prepare_registration(serial: &str) -> RegistrationData {
    let serial_hash: FixedBytes<32> = keccak256(serial.as_bytes());
    RegistrationData { serial_hash }
}

/// Verify a device reading against its on-chain registered address.
///
/// Returns `VerificationResult::Verified` only if the signature recovers to `on_chain_address`.
/// Pure: no I/O, no contract queries — the caller provides `on_chain_address`.
pub fn verify_device(reading: &Reading, on_chain_address: Address) -> VerificationResult {
    if on_chain_address == Address::ZERO {
        return VerificationResult::Unverified(UnverifiedReason::DeviceNotRegistered);
    }

    // Check if the signature is parseable before calling verify_reading
    let sig_hex = reading.signature.trim_start_matches("0x");
    let sig_parseable = hex::decode(sig_hex)
        .ok()
        .and_then(|b| AlloySignature::from_raw(b.as_slice()).ok())
        .is_some();

    if !sig_parseable {
        return VerificationResult::Unverified(UnverifiedReason::SignatureInvalid);
    }

    if verify_reading(reading, on_chain_address) {
        VerificationResult::Verified
    } else {
        VerificationResult::Unverified(UnverifiedReason::SignerMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hardtrust_core::{public_key_to_address, sign_reading};
    use k256::ecdsa::SigningKey;

    fn test_signing_key() -> SigningKey {
        let key_bytes =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("valid hex");
        SigningKey::from_slice(&key_bytes).expect("valid key")
    }

    fn signed_reading() -> (Reading, Address) {
        let key = test_signing_key();
        let address = public_key_to_address(key.verifying_key());
        let mut reading = Reading {
            serial: "TEST-001".to_string(),
            address: format!("{}", address),
            temperature: 22.5,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            signature: String::new(),
        };
        reading.signature = sign_reading(&key, &reading);
        (reading, address)
    }

    #[test]
    fn prepare_registration_produces_correct_serial_hash() {
        let data = prepare_registration("HARDCODED-001");
        let expected: FixedBytes<32> = keccak256(b"HARDCODED-001");
        assert_eq!(data.serial_hash, expected);
    }

    #[test]
    fn verify_device_returns_verified_for_valid_reading() {
        let (reading, address) = signed_reading();
        assert!(matches!(verify_device(&reading, address), VerificationResult::Verified));
    }

    #[test]
    fn verify_device_returns_signer_mismatch_for_wrong_address() {
        let (reading, _) = signed_reading();
        let wrong_address: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
            .parse()
            .unwrap();
        assert!(matches!(
            verify_device(&reading, wrong_address),
            VerificationResult::Unverified(UnverifiedReason::SignerMismatch)
        ));
    }

    #[test]
    fn verify_device_returns_not_registered_for_zero_address() {
        let (reading, _) = signed_reading();
        assert!(matches!(
            verify_device(&reading, Address::ZERO),
            VerificationResult::Unverified(UnverifiedReason::DeviceNotRegistered)
        ));
    }

    #[test]
    fn verify_device_returns_signature_invalid_for_fake_sig() {
        let (mut reading, address) = signed_reading();
        reading.signature = "0xFAKESIG".to_string();
        assert!(matches!(
            verify_device(&reading, address),
            VerificationResult::Unverified(UnverifiedReason::SignatureInvalid)
        ));
    }

    #[test]
    fn verify_device_returns_signer_mismatch_for_tampered_temperature() {
        let (mut reading, address) = signed_reading();
        reading.temperature = 99.0;
        assert!(matches!(
            verify_device(&reading, address),
            VerificationResult::Unverified(UnverifiedReason::SignerMismatch)
        ));
    }
}
