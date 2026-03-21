# S2a.1-V1: Protocol — Generic Signing Infrastructure

**Type:** Refactor
**Story:** [S2a.1 Device Capture](../stories/slice-2/s2a.1-device-capture.md)
**Depends on:** Existing protocol crate

---

## Problem

`crypto.rs` is fully coupled to `Reading` — `sign_reading()`, `reading_prehash()`, `verify_reading()` all hardcode the Reading type. Adding `Capture` would force duplicating all signing logic.

## What to Build

Extract generic signing primitives in `protocol/src/crypto.rs`. Both `Reading` and `Capture` will use them.

### Step 1 — Add `Signable` trait in `protocol/src/crypto.rs`

```rust
/// Trait for types that can be cryptographically signed.
/// Implementors define their canonical prehash payload.
pub trait Signable {
    /// Build the canonical keccak256 hash for signing.
    /// Returns None if the data is invalid (bad address, bad timestamp, etc).
    fn prehash(&self) -> Option<[u8; 32]>;

    /// Return the signature field (for verification).
    fn signature_hex(&self) -> &str;
}
```

### Step 2 — Implement `Signable` for `Reading`

Move the body of `reading_prehash()` into `Reading::prehash()`. Keep `reading_prehash()` as a thin wrapper calling the trait for backwards compatibility.

```rust
impl Signable for Reading {
    fn prehash(&self) -> Option<[u8; 32]> {
        // existing reading_prehash logic moves here
    }
    fn signature_hex(&self) -> &str {
        &self.signature
    }
}
```

### Step 3 — Generic sign/verify functions

```rust
/// Sign any Signable type. Returns "0x" + hex(r || s || v).
pub fn sign<T: Signable>(key: &SigningKey, data: &T) -> Result<String, ProtocolError> {
    // validate via prehash
    let hash = data.prehash().ok_or_else(|| ProtocolError::InvalidPayload)?;
    // sign_prehash_recoverable + format (existing logic from sign_reading)
    ...
}

/// Verify any Signable type against an expected on-chain address.
pub fn verify<T: Signable>(data: &T, on_chain_address: Address) -> bool {
    // existing verify_reading logic, using data.prehash() and data.signature_hex()
    ...
}
```

### Step 4 — Backwards compatibility wrappers

Keep `sign_reading` and `verify_reading` as thin wrappers:

```rust
pub fn sign_reading(key: &SigningKey, reading: &Reading) -> Result<String, ProtocolError> {
    sign(key, reading)
}

pub fn verify_reading(reading: &Reading, addr: Address) -> bool {
    verify(reading, addr)
}
```

### Step 5 — Add `InvalidPayload` variant to `ProtocolError`

```rust
pub enum ProtocolError {
    InvalidAddress(String),
    InvalidTimestamp(String),
    InvalidPayload,          // ← new
    SigningFailed(String),
}
```

### Step 6 — Add `Capture` domain type in `protocol/src/domain.rs`

```rust
/// A file entry in a capture manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureFile {
    pub name: String,
    pub hash: String,  // "sha256:..."
    pub size: u64,
}

/// A signed capture produced by a device after executing an external command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capture {
    pub serial: String,
    pub address: String,
    pub timestamp: String,
    pub content_hash: String,  // "sha256:..." root hash of all files
    pub files: Vec<CaptureFile>,
    pub signature: String,
}
```

### Step 7 — Implement `Signable` for `Capture`

```rust
impl Signable for Capture {
    fn prehash(&self) -> Option<[u8; 32]> {
        let serial_hash: [u8; 32] = Keccak256::digest(self.serial.as_bytes()).into();
        let addr_str = self.address.trim_start_matches("0x");
        let address_bytes: [u8; 20] = hex::decode(addr_str).ok()?.try_into().ok()?;
        // content_hash is "sha256:<hex>" — extract the hex part
        let content_hex = self.content_hash.strip_prefix("sha256:")?;
        let content_bytes: [u8; 32] = hex::decode(content_hex).ok()?.try_into().ok()?;
        let ts = chrono::DateTime::parse_from_rfc3339(&self.timestamp).ok()?.timestamp() as u64;

        let mut preimage = Vec::with_capacity(104);
        preimage.extend_from_slice(&serial_hash);       // 32
        preimage.extend_from_slice(&address_bytes);      // 20
        preimage.extend_from_slice(&content_bytes);      // 32
        preimage.extend_from_slice(&ts.to_be_bytes());   // 8
        // total: 92 bytes

        Some(Keccak256::digest(&preimage).into())
    }

    fn signature_hex(&self) -> &str {
        &self.signature
    }
}
```

### Step 8 — Export new types from `lib.rs`

```rust
pub use crypto::{sign, verify, Signable};
pub use domain::{Capture, CaptureFile, Reading};
```

---

## Files

| File | Action |
|------|--------|
| `protocol/src/crypto.rs` | Add `Signable` trait, generic `sign`/`verify`, refactor `sign_reading`/`verify_reading` as wrappers |
| `protocol/src/domain.rs` | Add `Capture`, `CaptureFile` types |
| `protocol/src/error.rs` | Add `InvalidPayload` variant |
| `protocol/src/lib.rs` | Export new types |

## Tests (TDD — write first)

1. `sign_reading` still works (existing tests pass unchanged)
2. `verify_reading` still works (existing tests pass unchanged)
3. `sign<Capture>` + `verify<Capture>` round-trip
4. `Capture` with tampered `content_hash` fails verify
5. `Capture` with tampered `timestamp` fails verify
6. `Capture` serializes/deserializes via serde
7. `Capture::prehash()` returns None for invalid address
8. `Capture::prehash()` returns None for invalid content_hash format

## Validation Criteria

- [ ] All existing Reading tests pass without modification
- [ ] `sign`/`verify` generics work for both `Reading` and `Capture`
- [ ] No duplicated signing logic
- [ ] `just ci` passes
