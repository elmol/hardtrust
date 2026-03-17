# S1-Debt-V3: Dead Code & Contract Cleanup

**Type:** Refactor (tech debt)
**Story Reference:** N/A — tech debt cleanup before Slice 2
**Depends on:** None

---

## What to Build

### 1. Delete `contracts/src/Counter.sol`

Default Foundry template scaffold. Dead code with no references.

### 2. Delete `contracts/test/Counter.t.sol` (if exists)

Default Foundry template test file.

### 3. Fix `--force` phantom flag (`device/src/main.rs`)

Line ~73 prints `"Run with --force to regenerate."` but `--force` is not implemented in the CLI struct. Change the message to:

```rust
"Device already initialized. Delete {} to regenerate.", key_path.display()
```

This is honest about what the user needs to do and doesn't promise unimplemented functionality.

### 4. Remove unused `OWNER` from contract (`contracts/src/HardTrustRegistry.sol`)

`OWNER` is set in the constructor but never referenced by any function. Remove it to save deployment bytecode and avoid confusion. If access control is needed later (Slice 2+), it will be re-added with actual usage.

### 5. Clean stale comments in `protocol/src/dev_config.rs`

Line 2-3: Update or remove "replaced in S1b" comment since S1b is complete.

### 6. Remove unused `DEV_SERIAL` and `DEV_ADDRESS` from `protocol/src/dev_config.rs`

These constants are only used by `scripts/e2e-the-wire.sh`, not by Rust code. They should not be compiled into the protocol library. If the e2e script needs them, it should define them inline.

---

## Files touched

- `contracts/src/Counter.sol` (DELETE)
- `contracts/test/Counter.t.sol` (DELETE if exists)
- `device/src/main.rs` (one line change)
- `contracts/src/HardTrustRegistry.sol` (remove OWNER)
- `protocol/src/dev_config.rs` (remove unused constants, fix comments)

## What NOT to Build

- Do not add `--force` implementation — that's a feature, not debt cleanup
- Do not add new access control to the contract
- Do not change e2e script (it can hardcode its own values)

## TDD Order

1. **Test:** `just ci` passes before changes (baseline)
2. **Implement:** Apply changes above
3. **Validate:** `just ci` + `just e2e-the-wire` both pass, `forge test` passes

## Validation Criteria

- [ ] No `Counter.sol` in codebase
- [ ] No `--force` mentioned in device output
- [ ] No `OWNER` in `HardTrustRegistry.sol`
- [ ] No `DEV_SERIAL` or `DEV_ADDRESS` in `dev_config.rs`
- [ ] `forge test` passes
- [ ] `just ci` passes
- [ ] `just e2e-the-wire` passes
