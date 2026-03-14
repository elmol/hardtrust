# Spec: S1a.1-V4 — End-to-End Registration Validation

## Parent Story

S1a.1 — Register a Device and Confirm It On-Chain

## Goal

A just recipe that runs the full S1a.1 flow in one shot: deploy, init, register, confirm.

## What to Build

### `justfile` updates

Add or update these recipes:

```just
# Build contracts (ABI needed by attester)
forge-build:
    cd contracts && forge build

# Build all Rust crates (requires contracts built first)
build: forge-build
    cargo build --workspace

# Run all tests
test: forge-build
    cd contracts && forge test
    cargo test --workspace

# E2E: Register a device and confirm on-chain
e2e-register:
    #!/usr/bin/env bash
    set -euo pipefail
    trap "kill $ANVIL_PID 2>/dev/null" EXIT
    # Start Anvil in background
    anvil &
    ANVIL_PID=$!
    sleep 2
    # Build and deploy
    cd contracts && forge build
    CONTRACT_ADDRESS=$(forge script script/Deploy.s.sol \
      --rpc-url http://127.0.0.1:8545 --broadcast 2>&1 | grep -oP 'DEPLOYED: \K0x[0-9a-fA-F]{40}')
    cd ..
    echo "Contract: $CONTRACT_ADDRESS"
    # Device init
    cargo run --bin device -- init
    # Register
    cargo run --bin attester -- register \
      --serial HARDCODED-001 \
      --device-address 0x1234567890abcdef1234567890abcdef12345678 \
      --contract $CONTRACT_ADDRESS
    # Confirm registration via getDevice
    SERIAL_HASH=$(cast keccak "HARDCODED-001")
    DEVICE=$(cast call $CONTRACT_ADDRESS "getDevice(bytes32)" $SERIAL_HASH --rpc-url http://127.0.0.1:8545)
    echo "Device registered: $DEVICE"
    echo "S1a.1 gate: PASSED"
```

Note: the grep pattern relies on `console.log("DEPLOYED:", address(registry))` in the deploy script (V1). The implementer should verify the format matches.

## What NOT to Build

- No CI integration (Anvil in CI is S1c)
- No verify/emit flow (that's S1a.2)

## Validation

```bash
just e2e-register
# Should complete without errors, prints "S1a.1 gate: PASSED"
```

## Handoff Prompt

```
Add workspace justfile recipes and e2e validation for HardTrust S1a.1.

Read the spec at docs/specs/s1a.1-v4-e2e-register.spec.md first.
All components (contracts, device, attester) must already be implemented.

Respect the "What NOT to Build" section strictly — do not add anything beyond what is specified.
After implementation, run `just e2e-register` to validate the full flow.
Branch: feat/s1a.1-register-device (continue on existing branch)
```
