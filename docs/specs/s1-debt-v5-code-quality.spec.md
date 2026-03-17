# S1-Debt-V5: Dependency Optimization & Code Quality

**Type:** Refactor (tech debt)
**Story Reference:** N/A — tech debt cleanup before Slice 2
**Depends on:** None

---

## What to Build

### 1. Optimize `alloy` features (`attester/Cargo.toml`)

Replace `features = ["full"]` with only what's needed:

```toml
alloy = { version = "...", features = ["providers", "signers", "sol-types", "contract", "primitives"] }
```

Run `cargo build` after to verify no missing features. Add features back only if compilation fails.

### 2. Optimize `tokio` features (`attester/Cargo.toml`)

Replace `features = ["full"]` with:

```toml
tokio = { version = "...", features = ["rt-multi-thread", "macros"] }
```

### 3. Remove unused `serde` feature from `chrono` (`device/Cargo.toml`)

```toml
# Before:
chrono = { version = "...", features = ["serde"] }
# After:
chrono = "..."
```

### 4. Add `PartialEq` derive to `ProtocolError` (`protocol/src/error.rs`)

```rust
#[derive(Debug, PartialEq)]
pub enum ProtocolError { ... }
```

This enables direct `assert_eq!` in tests instead of `matches!()` workarounds.

### 5. Fix inconsistent error handling in `device/src/main.rs`

Line ~99: Replace `eprintln!` + `process::exit(1)` with `return Err(...)` to match the pattern used everywhere else in `run()`.

### 6. Document float precision concern (`protocol/src/crypto.rs`)

Add a comment at line ~30 explaining the `(temperature * 1000.0) as i64` pattern and that it must match any future Solidity verification. Do NOT change the logic — just document the concern for Slice 2 when on-chain verification is added.

```rust
// Scale temperature to millidegrees for deterministic hashing.
// IMPORTANT: This scaling must match the Solidity-side verification
// exactly when on-chain ecrecover is implemented (Slice 2).
```

---

## Files touched

- `attester/Cargo.toml` (alloy + tokio features)
- `device/Cargo.toml` (chrono feature)
- `protocol/src/error.rs` (PartialEq derive)
- `device/src/main.rs` (eprintln → return Err)
- `protocol/src/crypto.rs` (comment only)

## What NOT to Build

- Do not change the float scaling logic
- Do not add new error types
- Do not update Cargo.lock manually (let cargo handle it)

## TDD Order

1. **Test:** `just ci` passes before changes (baseline)
2. **Implement:** Apply all changes
3. **Validate:** `just ci` passes, `cargo build --workspace` succeeds, no new warnings

## Validation Criteria

- [ ] `alloy` does not use `features = ["full"]`
- [ ] `tokio` does not use `features = ["full"]`
- [ ] `chrono` has no `serde` feature in device/Cargo.toml
- [ ] `ProtocolError` derives `PartialEq`
- [ ] No `process::exit` in `device/src/main.rs` `run()` function
- [ ] Float scaling has documentation comment
- [ ] `just ci` passes
- [ ] `cargo build --workspace` succeeds
