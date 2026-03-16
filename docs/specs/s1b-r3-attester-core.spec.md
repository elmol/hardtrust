# Spec: S1b-R3 — Attester CLI/Core Separation

## Parent Story

S1b — Real Crypto (all stories implemented: S1b.1, S1b.2, S1b.3)

## Goal

Separate the attester binary into a thin CLI shell (main.rs) and testable business logic (lib.rs). The two operations -- register and verify -- each get their own function in lib.rs, callable and testable without blockchain, file I/O, or Anvil.

This is a PURE REFACTOR -- no new behavior, no new features.

## What to Build

---

### 1. New file: `attester/src/lib.rs`

Two public functions that encapsulate the attester's business logic, plus supporting types.

#### `prepare_registration`

```rust
pub fn prepare_registration(serial: &str) -> RegistrationData
```

- Computes `serial_hash`: `keccak256(serial.as_bytes())` as `FixedBytes<32>`
- Returns a `RegistrationData` struct containing `serial_hash: FixedBytes<32>`
- Pure: no blockchain interaction, no signing, no I/O

`RegistrationData` is a new public struct in `attester/src/lib.rs`.

#### `verify_device`

```rust
pub fn verify_device(reading: &Reading, on_chain_address: Address) -> VerificationResult
```

- Calls `hardtrust_core::verify_reading` internally
- Returns a `VerificationResult` enum instead of a plain `bool`
- Pure: no I/O, no contract queries -- the caller provides `on_chain_address`

`VerificationResult` is a new public enum:

```rust
pub enum VerificationResult {
    Verified,
    Unverified(UnverifiedReason),
}

pub enum UnverifiedReason {
    SignatureInvalid,
    DeviceNotRegistered,
    SignerMismatch,
}
```

Mapping logic:
- `on_chain_address == Address::ZERO` --> `Unverified(DeviceNotRegistered)`
- `hardtrust_core::verify_reading` returns `false` AND the signature cannot be parsed --> `Unverified(SignatureInvalid)`
- `hardtrust_core::verify_reading` returns `false` AND the signature is parseable but the recovered address does not match --> `Unverified(SignerMismatch)`
- `hardtrust_core::verify_reading` returns `true` --> `Verified`

Note: the distinction between SignatureInvalid and SignerMismatch depends on what `verify_reading` can report. If the current `verify_reading` API (which returns `bool`) does not allow distinguishing these two cases, collapse them into a single `Unverified(VerificationFailed)` reason, or add a signature-parseable check before calling `verify_reading`. The implementer should choose the simplest approach that does not require changing `hardtrust_core`.

---

### 2. Refactored: `attester/src/main.rs`

Becomes a thin CLI shell that handles ONLY:
- Clap argument parsing (unchanged interface)
- Blockchain interaction (Alloy provider, contract queries, tx sending)
- File I/O (reading reading.json)
- Calls to lib.rs functions for business logic
- Output to stdout

#### Register command flow

1. Parse CLI args: serial, device-address, contract (IO/CLI)
2. Call `prepare_registration(&serial)` --> `RegistrationData` (CORE)
3. Create Alloy provider + wallet (IO -- uses dev_config for now)
4. Send `registerDevice` tx with `serial_hash` + `device_address` (IO)
5. Print tx hash (IO)

#### Verify command flow

1. Read reading.json from file (IO)
2. Parse JSON --> `Reading` (IO)
3. Query contract: `getDevice(serial_hash)` --> `on_chain_address` (IO)
4. Call `verify_device(&reading, on_chain_address)` --> `VerificationResult` (CORE)
5. Print `VERIFIED` or `UNVERIFIED` based on result (IO)

The external behavior (CLI arguments, output format, exit codes) does not change.

---

### 3. Unit tests in `attester/src/lib.rs`

All tests use a known `SigningKey` to create test `Reading` values via `hardtrust_core::sign_reading`. No Anvil, no blockchain, no file I/O.

- `prepare_registration("HARDCODED-001")` produces the correct `serial_hash` (known test vector: `keccak256(b"HARDCODED-001")`)
- `verify_device` with valid reading + matching on-chain address --> `Verified`
- `verify_device` with valid reading + wrong on-chain address --> `Unverified(SignerMismatch)`
- `verify_device` with `Address::ZERO` --> `Unverified(DeviceNotRegistered)`
- `verify_device` with fake signature `"0xFAKESIG"` --> `Unverified(SignatureInvalid)`
- `verify_device` with tampered temperature --> `Unverified(SignerMismatch)`

---

### 4. Existing tests

- The `reading_json_round_trip` test moves to `lib.rs` if it tests business logic, or stays in `main.rs` if it tests CLI/IO. Implementer decides based on what the test actually exercises.
- The ignored integration test stays in `main.rs`.

---

## Output Files

| File | Change |
|------|--------|
| `attester/src/lib.rs` | New -- `RegistrationData`, `VerificationResult`, `prepare_registration()`, `verify_device()` + unit tests |
| `attester/src/main.rs` | Refactored -- thin shell calling lib.rs |

## What NOT to Build

- No traits for blockchain abstraction
- No mocking of Alloy provider
- No changes to `device/` (that is S1b-R2)
- No error handling changes (that is S1b-R4)
- No changes to `hardtrust-core`
- No changes to the smart contract
- `dev_config` stays hardcoded in main.rs for now

## Dependencies

- **Requires:** S1b-R1 (core rename) must be merged -- this spec imports from `hardtrust_core`
- **Does NOT require:** S1b-R2 (device core separation) -- device and attester are independent refactors

## Validation

```bash
cargo test -p attester         # unit tests pass (new) + existing tests pass
cargo test --workspace         # all workspace tests pass
just e2e-the-wire              # The Wire gate: PASSED
```

---

## Handoff Prompt

```
Refactor: separate attester CLI from business logic.

Read the spec at docs/specs/s1b-r3-attester-core.spec.md first.
S1b-R1 must be merged first (hardtrust-core rename).
This is a PURE REFACTOR — no behavior changes.

Create attester/src/lib.rs with prepare_registration() and verify_device() as pure functions.
verify_device returns a VerificationResult enum (Verified, Unverified with reason).
Refactor main.rs to be a thin CLI shell that calls lib.rs.
Add unit tests for both functions in lib.rs.

After implementation, run:
  cargo test -p attester
  cargo test --workspace
  just e2e-the-wire
Branch: refactor/s1b-r3-attester-core
```
