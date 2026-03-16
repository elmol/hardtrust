# Spec: S1b-R1 — Rename types/ to core/ + Extract reading_prehash

## Parent Story

S1b — Real Crypto (all stories implemented: S1b.1, S1b.2, S1b.3)

## Goal

Two tightly coupled refactor items before moving to S1c:
1. Extract the duplicated hash preimage construction into a single `reading_prehash()` function
2. Rename the `types/` crate to `core/` (package name: `hardtrust-core`)

This is a PURE REFACTOR — no new behavior, no new features.

## What to Build

---

### 1. Extract reading_prehash

The 68-byte canonical preimage construction (serial_hash || address_bytes || temperature_scaled || timestamp_u64 -> keccak256) is duplicated in 3 places in `types/src/lib.rs`: `sign_reading()`, `verify_reading()`, and tests.

Add a single public function:

```rust
/// Build the canonical keccak256 hash of a Reading's signing payload.
/// Returns None if the reading has invalid address hex or timestamp format.
pub fn reading_prehash(reading: &Reading) -> Option<[u8; 32]>
```

Refactor `sign_reading()` and `verify_reading()` to call `reading_prehash()` instead of inlining the hash construction.

**Unit test:** call `reading_prehash()` with a known Reading, assert the hash matches a precomputed expected value.

---

### 2. Rename types/ to core/

- Rename directory: `types/` -> `core/`
- Update `core/Cargo.toml`: `name = "hardtrust-core"`
- Update workspace `Cargo.toml`: members list
- Update `device/Cargo.toml`: dependency path + name
- Update `attester/Cargo.toml`: dependency path + name
- Update ALL `use hardtrust_types::` to `use hardtrust_core::` in `device/` and `attester/`
- Update `cargo test -p hardtrust-core` (was `hardtrust-types`)

---

## Output Files

| File | Change |
|------|--------|
| `core/Cargo.toml` | Renamed from types/, name = "hardtrust-core" |
| `core/src/lib.rs` | Add `reading_prehash()`, refactor `sign_reading` + `verify_reading` to use it |
| `core/src/dev_config.rs` | Unchanged, just moved with directory |
| `Cargo.toml` (workspace) | Update members |
| `device/Cargo.toml` | Update dependency |
| `attester/Cargo.toml` | Update dependency |
| `device/src/main.rs` | Update imports |
| `attester/src/main.rs` | Update imports |

---

## What NOT to Build

- No module splitting (reading.rs, crypto.rs) — keep lib.rs as-is for now
- No behavior changes
- No new features
- No error handling changes (that is S1b-R4)

---

## Validation

```bash
cargo test --workspace        # all tests pass
just e2e-the-wire             # The Wire gate: PASSED
cargo test -p hardtrust-core  # reading_prehash unit test passes
```

---

## Handoff Prompt

```
Refactor: extract reading_prehash and rename types/ to core/.

Read the spec at docs/specs/s1b-r1-core-rename.spec.md first.
This is a PURE REFACTOR — no behavior changes.

Two items:
1. Add reading_prehash() to lib.rs, refactor sign_reading + verify_reading to call it
2. Rename types/ directory to core/, update all Cargo.toml and imports

After implementation, run:
  cargo test --workspace
  just e2e-the-wire
Branch: refactor/s1b-r1-core-rename
```
