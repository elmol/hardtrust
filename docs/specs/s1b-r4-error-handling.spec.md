# Spec: S1b-R4 — Error Handling (Replace .expect() with Result)

## Parent Story

S1b — Real Crypto (stories S1b.1, S1b.2, S1b.3 all implemented)

## Goal

Replace all `.expect()` panics with proper `Result`-based error handling. When something goes wrong, the user sees a clear error message on stderr — not a panic backtrace.

This is a PURE REFACTOR — no new behavior for valid inputs. For invalid inputs: error message instead of panic.

## What to Build

---

### 1. CoreError enum in `core/src/lib.rs`

Add a simple error enum:

```rust
#[derive(Debug)]
pub enum CoreError {
    InvalidAddress(String),
    InvalidTimestamp(String),
    SigningFailed(String),
}
impl std::fmt::Display for CoreError { ... }
impl std::error::Error for CoreError {}
```

Change `sign_reading` signature to return `Result`:

```rust
pub fn sign_reading(key: &SigningKey, reading: &Reading) -> Result<String, CoreError>
```

`verify_reading` already returns `bool` for invalid cases — no change needed.
`reading_prehash` already returns `Option` — no change needed.

---

### 2. Device binary: `device/src/main.rs`

Wrap `main()` in a `run()` function that returns `Result`:

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // ... all logic with ? operator
}
```

Replace every `.expect()` with `?` and human-readable error messages:

| Current `.expect()` | Replacement error message |
|---|---|
| HOME not set | "HOME environment variable not set" |
| Failed to create directory | "could not create ~/.hardtrust directory" |
| Failed to write key | "could not write device key" |
| Failed to set permissions | "could not set key file permissions" |
| Invalid key hex | "device.key contains invalid key data" |

Key file not found is already handled (eprintln + exit) — no change needed there.

---

### 3. Device library: `device/src/lib.rs`

`sign_reading` call now uses `?` since it returns `Result`. No other changes needed — lib.rs functions are already pure.

---

### 4. Attester binary: `attester/src/main.rs`

Same `run()` -> `Result` pattern:

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // ... all logic with ? operator
}
```

Replace every `.expect()` with `?` and human-readable error messages:

| Current `.expect()` | Replacement error message |
|---|---|
| Invalid private key | "invalid attester private key" |
| Invalid RPC URL | "invalid RPC URL" |
| Failed to read file | "could not read reading file: {path}" |
| Failed to parse JSON | "invalid reading JSON: {details}" |
| Contract query failed | "contract query failed: {details}" |
| Registration failed | "registration transaction failed: {details}" |

---

### 5. Attester library: `attester/src/lib.rs`

Minor: handle `sign_reading` `Result` if called. No other changes needed.

---

### 6. Unit tests to add

- **core:** `sign_reading` with invalid address hex returns `Err(CoreError::InvalidAddress)`
- **core:** `sign_reading` with invalid timestamp returns `Err(CoreError::InvalidTimestamp)`
- **device:** integration test: run `emit` without key file -> exit code 1 + error message on stderr (not panic)
- **attester:** integration test: run `verify` with nonexistent file -> exit code 1 + error message on stderr (not panic)

---

## Output Files

| File | Change |
|------|--------|
| `core/src/lib.rs` | CoreError enum, sign_reading returns Result |
| `device/src/main.rs` | run() -> Result pattern, replace .expect() |
| `device/src/lib.rs` | Propagate Result from sign_reading |
| `attester/src/main.rs` | run() -> Result pattern, replace .expect() |
| `attester/src/lib.rs` | Minor: handle sign_reading Result if called |

---

## Dependencies

- S1b-R1 must be merged (hardtrust-core crate exists)
- S1b-R2 must be merged (device/src/lib.rs exists)
- S1b-R3 must be merged (attester/src/lib.rs exists)

---

## What NOT to Build

- No custom error crate
- No anyhow or eyre dependency — `Box<dyn Error>` is sufficient for CLIs this size
- No recovery/retry logic
- No logging framework
- No changes to the contract

---

## Validation

```bash
cargo test --workspace         # all tests pass
just e2e-the-wire              # The Wire gate: PASSED
# Manual error path tests:
rm ~/.hardtrust/device.key && cargo run --bin device -- emit 2>&1 | grep -q "Error"
echo "bad json" > /tmp/bad.json && cargo run --bin attester -- verify --file /tmp/bad.json --contract 0x0 2>&1 | grep -q "Error"
```

---

## Handoff Prompt

```
Refactor: replace all .expect() panics with proper error handling.

Read the spec at docs/specs/s1b-r4-error-handling.spec.md first.
S1b-R1, S1b-R2, and S1b-R3 must be merged first.
This is a PURE REFACTOR — no behavior changes for valid inputs.

Add CoreError enum to hardtrust-core.
Wrap main() in run() -> Result pattern in both binaries.
Replace every .expect() with ? and human-readable error messages.
Users should see "Error: <message>" on stderr, not panic backtraces.

After implementation, run:
  cargo test --workspace
  just e2e-the-wire
Branch: refactor/s1b-r4-error-handling
```
