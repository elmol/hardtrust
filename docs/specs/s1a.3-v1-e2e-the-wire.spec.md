# Spec: S1a.3-V1 — E2E The Wire

## Parent Story

S1a.3 — Verify Data from an Unregistered Device

## Goal

A just recipe that runs the complete walking skeleton in one shot: deploy, register, emit, verify VERIFIED, then verify UNVERIFIED with a fake unregistered device. This is the FINAL gate of "The Wire" — proving the core value proposition that registered devices are VERIFIED and unregistered ones are UNVERIFIED.

## What to Build

### `justfile` updates

Add this recipe (do NOT modify existing `e2e-register` or `e2e-verify`):

```just
# E2E: The Wire — complete walking skeleton gate
e2e-the-wire:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill $ANVIL_PID 2>/dev/null' EXIT

    # Start Anvil
    anvil &
    ANVIL_PID=$!
    sleep 2

    # Build everything
    cd contracts && forge build && cd ..
    cargo build --workspace

    # Deploy contract
    DEPLOY_OUTPUT=$(cd contracts && forge script script/Deploy.s.sol \
      --rpc-url http://127.0.0.1:8545 --broadcast 2>&1)
    CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | awk '/DEPLOYED:/ {for(i=1;i<=NF;i++) if($i ~ /^0x/) print $i}')
    echo "Contract: $CONTRACT_ADDRESS"

    # Device init
    cargo run --bin device -- init

    # Register device
    cargo run --bin attester -- register \
      --serial HARDCODED-001 \
      --device-address 0x1234567890abcdef1234567890abcdef12345678 \
      --contract "$CONTRACT_ADDRESS"

    # Device emit
    cargo run --bin device -- emit
    echo "Reading written"

    # === CASE 1: VERIFIED (registered device) ===
    VERIFY_OUTPUT=$(cargo run --bin attester -- verify \
      --file reading.json \
      --contract "$CONTRACT_ADDRESS")
    echo "$VERIFY_OUTPUT"

    if [[ "$VERIFY_OUTPUT" != *"VERIFIED"* ]]; then
        echo "The Wire gate: FAILED — expected VERIFIED for registered device"
        exit 1
    fi
    echo "Case 1: VERIFIED — OK"

    # === CASE 2: UNVERIFIED (unregistered device) ===
    cat > fake-reading.json <<'FAKEJSON'
    {
      "serial": "FAKE-DEVICE-999",
      "address": "0x0000000000000000000000000000000000000BAD",
      "temperature": 22.5,
      "timestamp": "2025-01-01T00:00:00Z",
      "signature": "0xFAKESIG"
    }
    FAKEJSON

    UNVERIFIED_OUTPUT=$(cargo run --bin attester -- verify \
      --file fake-reading.json \
      --contract "$CONTRACT_ADDRESS")
    echo "$UNVERIFIED_OUTPUT"

    if [[ "$UNVERIFIED_OUTPUT" == *"VERIFIED"* && "$UNVERIFIED_OUTPUT" != *"UNVERIFIED"* ]]; then
        echo "The Wire gate: FAILED — expected UNVERIFIED for unregistered device"
        exit 1
    fi
    echo "Case 2: UNVERIFIED — OK"

    # Cleanup
    rm -f fake-reading.json

    echo ""
    echo "The Wire gate: PASSED"
```

### Unit test check

The `is_verified` unit tests from S1a.2-V2 already cover the UNVERIFIED path:
- `is_verified` returns `false` when addresses differ
- `is_verified` returns `false` when `on_chain_addr` is `Address::ZERO` (unregistered device)

No new unit tests needed. If the implementor finds these tests missing, add them following the same pattern from S1a.2-V2.

### Output files

- Updated `justfile` (add `e2e-the-wire` recipe)

## What NOT to Build

- No new subcommands
- No modifications to verify logic (already handles UNVERIFIED)
- No tampered data detection (S1b)
- No CI integration (S1c)
- No modifications to existing `e2e-register` or `e2e-verify` recipes
- No new unit tests unless the S1a.2-V2 `is_verified` tests are found missing

## Dependencies

- S1a.1 (all V1-V4) must be implemented — device init, contract deploy, attester register
- S1a.2 (all V1-V3) must be implemented — device emit, attester verify, e2e-verify

## Validation

```bash
cargo test -p attester              # existing is_verified tests pass
just e2e-the-wire                   # prints "The Wire gate: PASSED"
```

## Handoff Prompt

```
Add the e2e-the-wire recipe to validate the complete walking skeleton.

Read the spec at docs/specs/s1a.3-v1-e2e-the-wire.spec.md first.
S1a.1 and S1a.2 must be fully implemented.

Output files:
- Updated justfile (add e2e-the-wire recipe)

The UNVERIFIED case uses a fake reading with serial "FAKE-DEVICE-999" and address 0x0000000000000000000000000000000000000BAD.
Respect the "What NOT to Build" section strictly.
After implementation, run `just e2e-the-wire` to validate.
Branch: feat/s1a.3-v1-e2e-the-wire
```
