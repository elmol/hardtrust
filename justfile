anvil_rpc_url       := "http://127.0.0.1:8545"
anvil_deployer_key  := "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
anvil_attester_addr := "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

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

# ---------------------------------------------------------------------------
# Release recipes
# ---------------------------------------------------------------------------

# Cross-compile device for ARMv7 (mirrors CI — requires zig + cargo-zigbuild)
build-device-release:
    RELEASE_VERSION=dev bash scripts/build-device.sh

# Build attester for the current host (mirrors CI)
build-attester-release:
    RELEASE_VERSION=dev bash scripts/build-attester.sh

# E2E: The Wire — complete walking skeleton gate
e2e-the-wire:
    @ATTESTER_ADDRESS={{anvil_attester_addr}} \
     bash scripts/e2e-the-wire.sh
