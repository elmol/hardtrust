use alloy_primitives::{keccak256, Address, Signature as AlloySignature, B256};
use k256::ecdsa::{SigningKey, VerifyingKey};
use sha3::{Digest, Keccak256};

use crate::domain::Reading;
use crate::error::ProtocolError;

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
pub fn sign_reading(key: &SigningKey, reading: &Reading) -> Result<String, ProtocolError> {
    // Validate address
    let addr_str = reading.address.trim_start_matches("0x");
    if hex::decode(addr_str)
        .ok()
        .and_then(|b| <[u8; 20]>::try_from(b).ok())
        .is_none()
    {
        return Err(ProtocolError::InvalidAddress(reading.address.clone()));
    }

    // Validate timestamp
    if chrono::DateTime::parse_from_rfc3339(&reading.timestamp).is_err() {
        return Err(ProtocolError::InvalidTimestamp(reading.timestamp.clone()));
    }

    let hash = reading_prehash(reading).expect("already validated above");

    let (sig, recovery_id) = key
        .sign_prehash_recoverable(hash.as_ref())
        .map_err(|e| ProtocolError::SigningFailed(e.to_string()))?;

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
