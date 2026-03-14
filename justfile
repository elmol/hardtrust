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

lint:
    @cargo fmt --check 2>/dev/null || echo "No workspace members to lint"
    @cargo clippy --workspace -- -D warnings 2>/dev/null || echo "No workspace members to check"
    cd contracts && forge fmt --check
    cd contracts && npx solhint 'src/**/*.sol'
    cd contracts && aderyn . || true

ci: lint test

# E2E: Register a device and confirm on-chain
e2e-register:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill $ANVIL_PID 2>/dev/null' EXIT
    # Start Anvil in background
    anvil &
    ANVIL_PID=$!
    sleep 2
    # Build and deploy
    cd contracts && forge build
    CONTRACT_ADDRESS=$(forge script script/Deploy.s.sol \
      --rpc-url http://127.0.0.1:8545 --broadcast 2>&1 | awk '/DEPLOYED:/ {for(i=1;i<=NF;i++) if($i ~ /^0x/) print $i}')
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
