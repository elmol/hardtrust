# Spec: S1b-R2 — Device CLI/Core Separation

## Parent Story

S1b — Real Crypto (stories S1b.1, S1b.2, S1b.3 all implemented)

## Goal

Separate the device binary into a CLI shell (main.rs) and business logic (lib.rs). The two operations — init and emit — each get their own pure function in lib.rs, testable without file I/O or subprocess invocation.

This is a PURE REFACTOR — no new behavior.

## What to Build

---

### 1. New file: `device/src/lib.rs`

Two pure functions (no std::fs, no env vars, no IO):

**`pub fn init_device(signing_key: &SigningKey) -> DeviceIdentity`**
- Takes an already-generated signing key
- Derives Ethereum address via `hardtrust_core::public_key_to_address`
- Returns a `DeviceIdentity` struct with: `address` (String), `key_hex` (String, 64-char lowercase hex of private key)
- `DeviceIdentity` is a new public struct in lib.rs

**`pub fn create_signed_reading(signing_key: &SigningKey, serial: String, temperature: f64, timestamp: String) -> Reading`**
- Derives address from key
- Constructs a `Reading` struct
- Signs it via `hardtrust_core::sign_reading`
- Returns the complete Reading with real signature
- Pure: temperature and timestamp are PARAMETERS, not read from clock or sensors

---

### 2. Refactored: `device/src/main.rs`

Thin CLI shell that does ONLY:
- Clap parsing (unchanged)
- File I/O (read serial, read/write device.key, write reading.json)
- Calls lib.rs functions for the actual business logic
- Prints output

**Init command flow:**
1. Check if key exists (IO)
2. Read serial (IO)
3. Generate SigningKey with OsRng (IO — entropy source)
4. Call `init_device(&signing_key)` -> DeviceIdentity (CORE)
5. Write key_hex to file (IO)
6. Print serial + address (IO)

**Emit command flow:**
1. Read key from file (IO)
2. Read serial (IO)
3. Get timestamp from Utc::now() (IO)
4. Call `create_signed_reading(&key, serial, 22.5, timestamp)` (CORE)
5. Write reading.json (IO)

---

### 3. Unit tests in `device/src/lib.rs`

- `init_device` with a known SigningKey produces the correct address (compare with `public_key_to_address`)
- `init_device` key_hex round-trips (hex decode -> SigningKey -> same address)
- `create_signed_reading` produces a valid signature (verify with `hardtrust_core::verify_reading`)
- `create_signed_reading` with specific temperature + timestamp produces deterministic output

Existing integration tests in main.rs stay unchanged — they test the full binary via subprocess.

---

## Output Files

| File | Change |
|------|--------|
| `device/src/lib.rs` | New — DeviceIdentity struct, init_device(), create_signed_reading() + unit tests |
| `device/src/main.rs` | Refactored — thin shell calling lib.rs |

---

## What NOT to Build

- No traits or interfaces
- No changes to attester/ (that is S1b-R3)
- No error handling changes (that is S1b-R4)
- No changes to hardtrust-core
- No changes to `read_serial()` — it stays in main.rs (it is IO)
- No new crates
- No new features — pure refactor

---

## Dependencies

- S1b-R1 must be merged (imports from hardtrust_core, not hardtrust_types)

---

## Validation

```bash
cargo test -p device           # unit tests pass (new) + integration tests pass (existing)
cargo test --workspace         # all workspace tests pass
just e2e-the-wire              # The Wire gate: PASSED
```

---

## Handoff Prompt

```
Refactor: separate device CLI from business logic.

Read the spec at docs/specs/s1b-r2-device-core.spec.md first.
S1b-R1 must be merged first (hardtrust-core rename).
This is a PURE REFACTOR — no behavior changes.

Create device/src/lib.rs with init_device() and create_signed_reading() as pure functions.
Refactor main.rs to be a thin CLI shell that calls lib.rs.
Add unit tests for both functions in lib.rs.

After implementation, run:
  cargo test -p device
  cargo test --workspace
  just e2e-the-wire
Branch: refactor/s1b-r2-device-core
```
