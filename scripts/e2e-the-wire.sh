#!/usr/bin/env bash
set -euo pipefail
trap 'kill $ANVIL_PID 2>/dev/null' EXIT

# Start Anvil
anvil &
ANVIL_PID=$!
sleep 2

# Attester configuration — Anvil account #1 (well-known dev key)
export HARDTRUST_RPC_URL="http://127.0.0.1:8545"
export HARDTRUST_PRIVATE_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"

# Build everything
cd contracts && forge build && cd ..
cargo build --workspace

# Deploy contract (ATTESTER_ADDRESS passed from justfile)
DEPLOY_OUTPUT=$(cd contracts && ATTESTER_ADDRESS="${ATTESTER_ADDRESS}" forge script script/Deploy.s.sol \
  --rpc-url http://127.0.0.1:8545 --broadcast 2>&1)
CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | awk '/DEPLOYED:/ {for(i=1;i<=NF;i++) if($i ~ /^0x/) print $i}')
echo "Contract: $CONTRACT_ADDRESS"

# Device init — remove any existing key so init always prints fresh Serial and Address
rm -f "${HOME}/.hardtrust/device.key"
INIT_OUTPUT=$(cargo run --bin device -- init)
echo "$INIT_OUTPUT"
REAL_SERIAL=$(echo "$INIT_OUTPUT" | awk '/^Serial:/ {print $2}')
REAL_ADDRESS=$(echo "$INIT_OUTPUT" | awk '/^Address:/ {print $2}')
echo "Real serial:  $REAL_SERIAL"
echo "Real address: $REAL_ADDRESS"

# Register the real device identity on-chain
cargo run --bin attester -- register \
  --serial "${REAL_SERIAL}" \
  --device-address "${REAL_ADDRESS}" \
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

# === CASE 3: VERIFIED capture (registered device) ===
echo ""
echo "=== Case 3: Capture VERIFIED ==="

# Clean any previous capture output
rm -rf capture-output capture.json

cargo run --bin device -- capture \
  --cmd "./scripts/mock-capture.sh" \
  --output-dir ./capture-output

echo "Capture written"

VERIFY_CAPTURE=$(cargo run --bin attester -- verify \
  --file capture.json \
  --contract "$CONTRACT_ADDRESS")
echo "$VERIFY_CAPTURE"

if [[ "$VERIFY_CAPTURE" != *"VERIFIED"* ]]; then
    echo "The Wire gate: FAILED — expected VERIFIED for capture from registered device"
    exit 1
fi
echo "Case 3: Capture VERIFIED — OK"

# === CASE 4: UNVERIFIED capture (fake capture) ===
echo ""
echo "=== Case 4: Capture UNVERIFIED ==="

cat > fake-capture.json <<'FAKEJSON'
{
  "serial": "FAKE-SCOPE-999",
  "address": "0x0000000000000000000000000000000000000BAD",
  "timestamp": "2025-01-01T00:00:00Z",
  "content_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "files": [
    { "name": "fake.jpg", "hash": "sha256:1111111111111111111111111111111111111111111111111111111111111111", "size": 100 }
  ],
  "environment": {
    "script_hash": "sha256:fake",
    "binary_hash": "sha256:fake",
    "hw_serial": "FAKE-HW",
    "camera_info": "fake-camera"
  },
  "signature": "0xFAKESIG"
}
FAKEJSON

UNVERIFIED_CAPTURE=$(cargo run --bin attester -- verify \
  --file fake-capture.json \
  --contract "$CONTRACT_ADDRESS")
echo "$UNVERIFIED_CAPTURE"

if [[ "$UNVERIFIED_CAPTURE" == *"VERIFIED"* && "$UNVERIFIED_CAPTURE" != *"UNVERIFIED"* ]]; then
    echo "The Wire gate: FAILED — expected UNVERIFIED for fake capture"
    exit 1
fi
echo "Case 4: Capture UNVERIFIED — OK"

# === CASE 5: Environment MATCH with --release-hashes ===
echo ""
echo "=== Case 5: Environment MATCH ==="

# Build SHA256SUMS from the actual capture
SCRIPT_HASH=$(sha256sum ./scripts/mock-capture.sh | awk '{print "sha256:" $1}')
BINARY_HASH=$(jq -r '.environment.binary_hash' capture.json)
cat > SHA256SUMS <<EOF
${SCRIPT_HASH}  terrascope-capture.sh
${BINARY_HASH}  device-armv7
EOF

VERIFY_ENV=$(cargo run --bin attester -- verify \
  --file capture.json \
  --contract "$CONTRACT_ADDRESS" \
  --release-hashes SHA256SUMS)
echo "$VERIFY_ENV"

if [[ "$VERIFY_ENV" != *"MATCH"* ]]; then
    echo "The Wire gate: FAILED — expected MATCH for environment hashes"
    exit 1
fi
echo "Case 5: Environment MATCH — OK"

# === CASE 6: Environment MISMATCH (tampered script) ===
echo ""
echo "=== Case 6: Environment MISMATCH ==="

# Create a tampered capture script
cp ./scripts/mock-capture.sh /tmp/tampered-capture.sh
echo "# tampered" >> /tmp/tampered-capture.sh
chmod +x /tmp/tampered-capture.sh

rm -rf capture-output capture.json
cargo run --bin device -- capture \
  --cmd "/tmp/tampered-capture.sh" \
  --output-dir ./capture-output

VERIFY_TAMPERED=$(cargo run --bin attester -- verify \
  --file capture.json \
  --contract "$CONTRACT_ADDRESS" \
  --release-hashes SHA256SUMS)
echo "$VERIFY_TAMPERED"

if [[ "$VERIFY_TAMPERED" != *"VERIFIED"* ]]; then
    echo "The Wire gate: FAILED — tampered capture should still have VERIFIED signature"
    exit 1
fi
if [[ "$VERIFY_TAMPERED" != *"MISMATCH"* ]]; then
    echo "The Wire gate: FAILED — expected MISMATCH for tampered script hash"
    exit 1
fi
echo "Case 6: Environment MISMATCH (signature still VERIFIED) — OK"

# Cleanup
rm -f fake-reading.json fake-capture.json capture.json reading.json SHA256SUMS
rm -rf capture-output
rm -f /tmp/tampered-capture.sh

echo ""
echo "The Wire gate: PASSED (6 cases — reading verified/unverified + capture verified/unverified + env match/mismatch)"
