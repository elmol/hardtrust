# S2a.1-V3: Attester — Extend Verify to Support Captures

**Type:** Feature (extension)
**Story:** [S2a.1 Device Capture](../stories/slice-2/s2a.1-device-capture.md)
**Depends on:** S2a.1-V1 (Signable trait, generic verify), S2a.1-V2 (Capture type in use)

---

## Problem

`attester verify` only accepts `reading.json` (Reading type). It cannot verify `capture.json` files produced by `device capture`. Both types share the same verification flow (recover signer → check on-chain registry), and the `Signable` trait + generic `verify<T>` from V1 already handle the crypto. The attester just needs to detect the file format and dispatch.

## What to Build

### Step 1 — Add generic `verify_device_data` in `attester/src/lib.rs`

The existing `verify_device` is coupled to `Reading`. Add a generic version using `Signable`:

```rust
use hardtrust_protocol::Signable;

/// Verify any Signable data against an on-chain address.
pub fn verify_device_data<T: Signable>(data: &T, on_chain_address: Address) -> VerificationResult {
    if on_chain_address == Address::ZERO {
        return VerificationResult::Unverified(UnverifiedReason::DeviceNotRegistered);
    }

    let sig_hex = data.signature_hex().trim_start_matches("0x");
    let sig_parseable = hex::decode(sig_hex)
        .ok()
        .and_then(|b| AlloySignature::from_raw(b.as_slice()).ok())
        .is_some();

    if !sig_parseable {
        return VerificationResult::Unverified(UnverifiedReason::SignatureInvalid);
    }

    if hardtrust_protocol::verify(data, on_chain_address) {
        VerificationResult::Verified
    } else {
        VerificationResult::Unverified(UnverifiedReason::SignerMismatch)
    }
}
```

Keep `verify_device` as a backwards-compatible wrapper:

```rust
pub fn verify_device(reading: &Reading, on_chain_address: Address) -> VerificationResult {
    verify_device_data(reading, on_chain_address)
}
```

### Step 2 — Auto-detect file format in `attester/src/main.rs`

The `Verify` command currently hardcodes `Reading`. Change it to detect the format:

```rust
Command::Verify { file, contract } => {
    let json = std::fs::read_to_string(&file)
        .map_err(|e| format!("could not read file {file}: {e}"))?;

    // Detect format: Capture has "content_hash", Reading has "temperature"
    let (serial, result) = if json.contains("content_hash") {
        let capture: Capture = serde_json::from_str(&json)
            .map_err(|e| format!("invalid capture JSON: {e}"))?;
        let reg = prepare_registration(&capture.serial);
        let provider = ProviderBuilder::new().connect_http(env_rpc_url()?);
        let registry = HardTrustRegistry::new(contract, &provider);
        let device = registry.getDevice(reg.serial_hash).call().await
            .map_err(|e| format!("contract query failed: {e}"))?;
        (capture.serial.clone(), verify_device_data(&capture, device.deviceAddr))
    } else {
        let reading: Reading = serde_json::from_str(&json)
            .map_err(|e| format!("invalid reading JSON: {e}"))?;
        let reg = prepare_registration(&reading.serial);
        let provider = ProviderBuilder::new().connect_http(env_rpc_url()?);
        let registry = HardTrustRegistry::new(contract, &provider);
        let device = registry.getDevice(reg.serial_hash).call().await
            .map_err(|e| format!("contract query failed: {e}"))?;
        (reading.serial.clone(), verify_device_data(&reading, device.deviceAddr))
    };

    match result {
        VerificationResult::Verified => println!("VERIFIED"),
        VerificationResult::Unverified(_) => println!("UNVERIFIED"),
    }
}
```

### Step 3 — Update imports in `attester/src/main.rs`

```rust
use hardtrust_protocol::{Capture, Reading};
use attester::{verify_device_data, ...};
```

---

## Files

| File | Action |
|------|--------|
| `attester/src/lib.rs` | Add `verify_device_data<T: Signable>`, keep `verify_device` as wrapper |
| `attester/src/main.rs` | Auto-detect Reading vs Capture in Verify command, import Capture |

## Tests (TDD — write first)

### In `attester/src/lib.rs`:

1. `verify_device_data` with valid signed Capture → Verified
2. `verify_device_data` with tampered capture content_hash → SignerMismatch
3. `verify_device_data` with zero address → DeviceNotRegistered
4. `verify_device_data` with fake signature → SignatureInvalid
5. Existing `verify_device` (Reading) tests pass unchanged

### In `attester/src/main.rs` (integration):

6. `attester verify --file capture.json` with bad JSON → clear error
7. `attester verify --file capture.json` with missing fields → clear error

## Validation Criteria

- [ ] `attester verify --file reading.json --contract 0x...` still works (Reading)
- [ ] `attester verify --file capture.json --contract 0x...` works (Capture)
- [ ] Auto-detection is correct (no new CLI flag needed)
- [ ] All existing tests pass unchanged
- [ ] `just ci` passes
