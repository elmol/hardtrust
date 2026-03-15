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

# Deploy contract (ATTESTER_ADDRESS passed from justfile)
DEPLOY_OUTPUT=$(cd contracts && ATTESTER_ADDRESS="${ATTESTER_ADDRESS}" forge script script/Deploy.s.sol \
  --rpc-url http://127.0.0.1:8545 --broadcast 2>&1)
CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | awk '/DEPLOYED:/ {for(i=1;i<=NF;i++) if($i ~ /^0x/) print $i}')
echo "Contract: $CONTRACT_ADDRESS"

# Device init
cargo run --bin device -- init

# Register device
cargo run --bin attester -- register \
  --serial "${DEV_SERIAL}" \
  --device-address "${DEV_ADDRESS}" \
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
