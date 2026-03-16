# Spec: S1a Refactor — Polish The Wire

## Parent Story

S1a — The Wire (walking skeleton, all gates passed: S1a.1, S1a.2, S1a.3)

## Goal

Polish The Wire before S1b (Real Crypto). Pure refactor — no new behavior, no new features. All existing e2e gates must continue to pass after this change.

## What to Build

This spec covers five refactor items. All changes are purely cosmetic, structural, or organizational. No production logic is modified.

---

### 1. README Update

Update `/README.md` to document:

- What HardTrust is (expand the existing one-liner slightly)
- The Wire: what it proves and how to run it
- Prerequisites: Foundry, Rust, Anvil
- Quick start: `just build`, `just test`, `just e2e-the-wire`
- Architecture overview: `contracts/`, `device/`, `attester/`, `types/`

---

### 2. `--help` Improvements (clap)

Improve the `--help` output for all CLI commands. Update the clap `about`/`long_about` and argument `help` strings only — do not change argument names, types, or behavior.

`device init`
- Describe what the command prints (device serial, Ethereum address) and why (proves deterministic key derivation from serial).

`device emit`
- Describe what `reading.json` contains (mock sensor reading with serial and timestamp).

`attester register`
- `--serial`: the device's unique serial number
- `--device-address`: the Ethereum address derived from the device serial (output of `device init`)
- `--contract`: the deployed DeviceRegistry contract address

`attester verify`
- `--file`: path to the reading.json file produced by `device emit`
- `--contract`: the deployed DeviceRegistry contract address
- Describe what VERIFIED/UNVERIFIED means in the output.

---

### 3. Consolidate e2e Scripts

Remove `e2e-register` and `e2e-verify` recipes from the justfile entirely. These recipes are deleted — not moved, not renamed.

Keep ONLY `e2e-the-wire` as the single comprehensive e2e gate. It already covers:
- Full flow: register + emit + verify
- Both outcomes: VERIFIED and UNVERIFIED

---

### 4. Extract `e2e-the-wire` to a Script

Move the bash content of the `e2e-the-wire` justfile recipe to `scripts/e2e-the-wire.sh`.

The justfile recipe becomes:

```just
e2e-the-wire:
    @bash scripts/e2e-the-wire.sh
```

The script must:
- Be placed at `scripts/e2e-the-wire.sh`
- Be executable (`chmod +x`)
- Contain exactly the same logic as the current inline recipe
- Use env vars or arguments for any values sourced from justfile variables (see item 5)

---

### 5. Centralize Configuration Constants

**Rust side — new file `types/src/dev_config.rs`:**

```rust
// Dev-only constants for The Wire (hardcoded Anvil values).
// These will be replaced with real configuration in S1b.
pub const DEV_SERIAL: &str = "HARDCODED-001";
pub const DEV_ADDRESS: &str = "0x1234567890abcdef1234567890abcdef12345678";
pub const DEV_RPC_URL: &str = "http://127.0.0.1:8545";
pub const DEV_ATTESTER_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"; // Anvil #1 — attester signing key (NOT the deployer)
```

Export from `types/src/lib.rs`:

```rust
pub mod dev_config;
```

Update `device/src/main.rs` and `attester/src/main.rs` to import and use these constants instead of any locally defined equivalents. Note: `DEV_ATTESTER_KEY` is used only in `attester/src/main.rs` (the register subcommand signer). Do not use it in the deploy script.

**justfile side — add variables at the top:**

```just
anvil_rpc_url        := "http://127.0.0.1:8545"
anvil_deployer_key   := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
anvil_attester_addr  := "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
dev_serial           := "HARDCODED-001"
dev_address          := "0x1234567890abcdef1234567890abcdef12345678"
```

Reference these via `{{anvil_rpc_url}}` etc. in recipes. Pass to `e2e-the-wire.sh` as environment variables or arguments.

**Solidity side — `contracts/script/Deploy.s.sol`:**

Update the deploy script to read the attester address from an environment variable instead of a hardcoded literal:

```solidity
address attester = vm.envAddress("ATTESTER_ADDRESS");
```

Pass from the justfile deploy recipe:

```just
ATTESTER_ADDRESS={{anvil_attester_addr}} forge script ...
```

---

## Output Files

| File | Change |
|---|---|
| `README.md` | Updated |
| `types/src/dev_config.rs` | New file |
| `types/src/lib.rs` | Add `pub mod dev_config` |
| `device/src/main.rs` | Use `dev_config` constants + `--help` improvements |
| `attester/src/main.rs` | Use `dev_config` constants + `--help` improvements |
| `justfile` | Add variables, remove `e2e-register`/`e2e-verify`, simplify `e2e-the-wire` |
| `scripts/e2e-the-wire.sh` | Extracted from justfile |
| `contracts/script/Deploy.s.sol` | Use env var for attester address |

---

## What NOT to Build

- No runtime config loading (no `.env` files, no `dotenv` crate, no TOML config)
- No new features — pure refactor
- No changes to production logic (`verify`, `register`, `emit` behavior unchanged)
- No CI changes (that is S1c)
- No new crates
- `e2e-register` and `e2e-verify` are deleted, not moved

---

## Validation

```bash
just build          # compiles without errors
just test           # all tests pass
just e2e-the-wire   # The Wire gate: PASSED
device --help       # shows improved help text
attester --help     # shows improved help text
```

The final line of `just e2e-the-wire` must still print `The Wire gate: PASSED`.

---

## Handoff Prompt

```
Refactor HardTrust "The Wire" polish. This is a PURE REFACTOR — no new behavior, no new features.

Read the spec at docs/specs/s1a-refactor.spec.md first.
All S1a gates (S1a.1, S1a.2, S1a.3) have already passed. Do not modify production logic.

Work through the five items in order:
1. README update
2. --help improvements (clap strings only)
3. Remove e2e-register and e2e-verify recipes (delete entirely)
4. Extract e2e-the-wire to scripts/e2e-the-wire.sh
5. Centralize constants into types/src/dev_config.rs and justfile variables

After implementation, validate with:
  just build
  just test
  just e2e-the-wire   # must print: The Wire gate: PASSED

Respect the "What NOT to Build" section strictly.
Branch: refactor/s1a-the-wire
```
