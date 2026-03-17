# S1-Debt-V2: CI Pipeline Fix & Docs Sync

**Type:** Refactor (tech debt)
**Story Reference:** N/A — tech debt cleanup before Slice 2
**Depends on:** S1-Debt-V1 (Implemented)

---

## What to Build

### 1. Fix CI silent failure (`.github/workflows/ci.yml`)

The CI `test` job uses `2>/dev/null || echo "No workspace members to test"` which silently swallows compilation errors — CI can report green without running Rust tests.

**Changes:**

- **test job:** Add `forge build` step BEFORE `cargo test` (attester needs `contracts/out/` at compile time via `sol!` macro)
- **test job:** Remove `2>/dev/null || echo "No workspace members to test"` from `cargo test --workspace`
- **lint job:** Remove `2>/dev/null || echo` patterns from `cargo fmt --check` and `cargo clippy`
- **integration job:** If it's a stub, either remove it or document it as placeholder

Expected test job order:
```yaml
- name: Install Foundry
  uses: foundry-rs/foundry-toolchain@v1
- name: Build contracts
  run: forge build
- name: Run Rust tests
  run: cargo test --workspace
```

### 2. Fix `docs/architecture.md` stale references

Replace all references to `common/` crate with `protocol/`. Update workspace members from `device, attester, common` to `device, attester, protocol`.

### 3. Fix `README.md` minor issues

- Add `aderyn` to prerequisites (or note it as optional/local-only)
- Verify CI section accuracy after ci.yml changes

---

## Files touched

- `.github/workflows/ci.yml`
- `docs/architecture.md`
- `README.md`

## What NOT to Build

- Do not change any Rust source code
- Do not change Justfile
- Do not add new CI jobs

## TDD Order

1. **Test:** Push branch, verify CI runs and actually executes `cargo test` (check logs for test output)
2. **Implement:** Apply changes above
3. **Validate:** CI passes with real test output visible in logs (not swallowed)

## Validation Criteria

- [ ] `cargo test --workspace` runs in CI WITHOUT `2>/dev/null`
- [ ] `forge build` runs before `cargo test` in CI
- [ ] `docs/architecture.md` has zero references to `common/`
- [ ] CI logs show actual test results (not silent pass)
- [ ] `just ci` passes locally
