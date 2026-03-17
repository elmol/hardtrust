# S1-Debt-V1: Extract Registration Error Classification

**Type:** Refactor (tech debt)
**Story Reference:** N/A — tech debt cleanup before Slice 2
**Depends on:** All S1c specs (Implemented)

---

## What to Build

Extract the `DeviceAlreadyRegistered` error detection from `attester/src/main.rs` (lines 91–98) into a pure function in `attester/src/lib.rs`, unit test it, and remove the 3 `#[ignore]` tests that never run.

### 1. New function in `attester/src/lib.rs`

```rust
/// Classification of a registration transaction error.
pub enum RegistrationError {
    /// The device is already registered on-chain.
    AlreadyRegistered { serial_hash: String },
    /// Any other transaction failure.
    TransactionFailed(String),
}

/// Classify a registration error from its string representation.
///
/// Detects `DeviceAlreadyRegistered` custom error (selector `0xa98bbce0`)
/// from the Alloy error string. Pure: no I/O.
pub fn classify_registration_error(error: &str, serial_hash: &str) -> RegistrationError {
    if error.contains("DeviceAlreadyRegistered") || error.contains("a98bbce0") {
        RegistrationError::AlreadyRegistered {
            serial_hash: serial_hash.to_string(),
        }
    } else {
        RegistrationError::TransactionFailed(error.to_string())
    }
}
```

### 2. Update `attester/src/main.rs`

Replace the inline `.map_err(|e| { ... })` closure (lines 91–98) with a call to `classify_registration_error`, then format the result for display:

```rust
.map_err(|e| {
    match classify_registration_error(&format!("{e}"), &format!("{}", reg.serial_hash)) {
        RegistrationError::AlreadyRegistered { serial_hash } =>
            format!("device already registered (serial hash: {serial_hash})"),
        RegistrationError::TransactionFailed(msg) =>
            format!("registration transaction failed: {msg}"),
    }
})?
```

### 3. Remove `#[ignore]` tests from `main.rs`

Delete these 3 tests:

| Test | Line | Reason for removal |
|------|------|--------------------|
| `verify_registered_device` | 216 | Empty body — placeholder with no assertions. The actual verification logic is already tested in `lib.rs::tests` (5 tests). |
| `register_duplicate_shows_human_error` | 223 | Replace with unit test of `classify_registration_error` in `lib.rs`. The CLI formatting is trivial. |
| `register_success_shows_confirmation` | 263 | Tests stdout string "Registered device. tx:" — covered by `just e2e-the-wire` which runs the full flow. |

### 4. New unit tests in `attester/src/lib.rs`

Add to the existing `mod tests`:

```rust
#[test]
fn classify_error_detects_already_registered_by_name() {
    let err = "server returned an error response: DeviceAlreadyRegistered(0xabc...)";
    let result = classify_registration_error(err, "0x1234");
    assert!(matches!(result, RegistrationError::AlreadyRegistered { .. }));
}

#[test]
fn classify_error_detects_already_registered_by_selector() {
    let err = "execution reverted: custom error a98bbce0:00000...";
    let result = classify_registration_error(err, "0x1234");
    assert!(matches!(result, RegistrationError::AlreadyRegistered { .. }));
}

#[test]
fn classify_error_returns_transaction_failed_for_other_errors() {
    let err = "connection refused";
    let result = classify_registration_error(err, "0x1234");
    assert!(matches!(result, RegistrationError::TransactionFailed(_)));
}
```

---

## What NOT to Build

- Do not add new integration tests that require Anvil
- Do not change the CLI user-facing output (same error messages)
- Do not refactor verify logic — only register error classification
- Do not touch `protocol/` crate

## TDD Order

1. **Test:** Write the 3 `classify_registration_error` unit tests in `lib.rs` — they fail (function doesn't exist)
2. **Implement:** Add `RegistrationError` enum and `classify_registration_error` to `lib.rs`
3. **Refactor:** Update `main.rs` to call the new function, delete the 3 `#[ignore]` tests
4. **Validate:** `just ci` passes, no `#[ignore]` tests remain

## Validation Criteria

- [ ] `classify_registration_error` is a public pure function in `attester/src/lib.rs`
- [ ] 3 new unit tests pass without Anvil
- [ ] 0 `#[ignore]` tests in the codebase
- [ ] `main.rs` error handling delegates to `classify_registration_error`
- [ ] CLI output unchanged: "device already registered (serial hash: ...)" for duplicates
- [ ] `just ci` passes
- [ ] `just e2e-the-wire` passes
