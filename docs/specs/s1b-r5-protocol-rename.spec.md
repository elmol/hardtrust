# Spec: S1b-R5 — Rename core/ to protocol/ + Internal Modules

## Parent Story

S1b — Real Crypto (refactor continuation)

## Goal

Rename the `core/` crate to `protocol/` (`hardtrust-protocol`) and organize internal modules by concern. The crate defines the shared agreement between device and attester: what a Reading is, how it's hashed, and how it's signed/verified. "protocol" reflects this better than the generic "core".

Also create ADR-0008 documenting this architectural decision.

This is a PURE REFACTOR — no new behavior, no new features.

## What to Build

---

### 1. Rename core/ to protocol/

- Rename directory: `core/` -> `protocol/`
- Update `protocol/Cargo.toml`: `name = "hardtrust-protocol"`
- Update workspace `Cargo.toml`: members list
- Update `device/Cargo.toml`: dependency path + name
- Update `attester/Cargo.toml`: dependency path + name
- Update ALL `use hardtrust_core::` to `use hardtrust_protocol::` in `device/` and `attester/`

---

### 2. Internal modules

Split `protocol/src/lib.rs` into focused modules. Re-export everything from `lib.rs` so the public API stays flat — consumers keep `use hardtrust_protocol::{Reading, sign_reading, ...}`.

| Module | Contents | Moved from |
|--------|----------|------------|
| `domain.rs` | `Reading` struct (+ serde derives) | lib.rs |
| `crypto.rs` | `sign_reading()`, `verify_reading()`, `reading_prehash()`, `public_key_to_address()` | lib.rs |
| `error.rs` | `CoreError` → renamed to `ProtocolError` (with Display + Error impls) | lib.rs |
| `dev_config.rs` | Unchanged — stays as-is | already separate |

`lib.rs` becomes ~15 lines of `mod` declarations and `pub use` re-exports.

---

### 3. ADR-0008: Protocol Crate Naming

Create `docs/adrs/adr-0008-protocol-crate.md`:

- **Decision:** Shared library crate is named `hardtrust-protocol`, not `hardtrust-core`
- **Context:** The crate defines the protocol both parties agree on — Reading struct, canonical hashing, sign/verify. "core" is a generic dumping-ground name that doesn't communicate intent.
- **Alternatives rejected:**
  - `hardtrust-core` — too generic, becomes a catch-all
  - Multiple crates (hardtrust-crypto, hardtrust-domain) — over-engineered at this scale; sign/verify are tightly coupled to Reading via reading_prehash
  - Move sign→device, verify→attester — creates cross-dependency in tests (attester needs sign_reading for test fixtures)
- **Consequence:** Both device and attester depend on `hardtrust-protocol`. Internal modules organize by concern without crate-level splitting.

---

## Tests

All existing tests must pass unchanged (same public API, just re-exported from modules).

**TDD approach:** Since this is a pure refactor, run existing tests FIRST to establish the green baseline, then rename/reorganize, then verify tests still pass.

```bash
# Before: establish baseline
cargo test --workspace

# After each step: verify no regressions
cargo test -p hardtrust-protocol
cargo test --workspace
just e2e-the-wire
```

---

## Output Files

| File | Change |
|------|--------|
| `protocol/Cargo.toml` | Renamed from core/, name = "hardtrust-protocol" |
| `protocol/src/lib.rs` | Module declarations + re-exports only (~15 lines) |
| `protocol/src/domain.rs` | Reading struct |
| `protocol/src/crypto.rs` | sign_reading, verify_reading, reading_prehash, public_key_to_address |
| `protocol/src/error.rs` | ProtocolError enum (renamed from CoreError) |
| `protocol/src/dev_config.rs` | Unchanged, moved with directory |
| `Cargo.toml` (workspace) | Update members |
| `device/Cargo.toml` | Update dependency |
| `attester/Cargo.toml` | Update dependency |
| `device/src/lib.rs` | `use hardtrust_protocol::` |
| `device/src/main.rs` | `use hardtrust_protocol::` |
| `attester/src/lib.rs` | `use hardtrust_protocol::` |
| `attester/src/main.rs` | `use hardtrust_protocol::` |
| `docs/adrs/adr-0008-protocol-crate.md` | New ADR |

---

## What NOT to Build

- No new functions or behavior
- No dependency changes (same crates: k256, alloy-primitives, sha3, hex, chrono, serde)
- No changes to sign/verify logic
- No new crates — just rename + internal reorganization

---

## Validation

```bash
cargo test --workspace        # all tests pass
cargo test -p hardtrust-protocol  # protocol crate tests pass
just e2e-the-wire             # The Wire gate: PASSED
```

---

## Handoff Prompt

```
Refactor: rename core/ to protocol/ and split into internal modules.

Read the spec at docs/specs/s1b-r5-protocol-rename.spec.md first.
This is a PURE REFACTOR — no behavior changes.

Three items:
1. Rename core/ directory to protocol/, update all Cargo.toml and imports
   (hardtrust-core -> hardtrust-protocol, CoreError -> ProtocolError)
2. Split lib.rs into domain.rs, crypto.rs, error.rs modules.
   Re-export from lib.rs so public API stays flat.
3. Create ADR-0008 in docs/adrs/adr-0008-protocol-crate.md

TDD: run cargo test --workspace FIRST (green baseline), then refactor, then verify.

After implementation, run:
  cargo test --workspace
  just e2e-the-wire
Branch: refactor/s1b-r5-protocol-rename
```
