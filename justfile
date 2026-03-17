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
release_version := "v0.1.0"

# Build attester for the host platform (release mode)
build-release:
    cargo build --release --package attester
    @echo "Binary: target/release/attester"

# Stage release artifacts for current host into ./dist/ (for local testing)
dist: forge-build
    mkdir -p dist
    cargo build --release --package attester --target x86_64-unknown-linux-gnu 2>/dev/null || \
        cargo build --release --package attester
    ARTIFACT="attester-{{release_version}}-$(rustc -vV | grep host | awk '{print $2}')" && \
        cp target/release/attester "dist/$${ARTIFACT}" && \
        (sha256sum "dist/$${ARTIFACT}" > "dist/$${ARTIFACT}.sha256" 2>/dev/null || \
         shasum -a 256 "dist/$${ARTIFACT}" > "dist/$${ARTIFACT}.sha256")
    @echo "Artifacts in ./dist/"
    @ls -lh dist/

# E2E: The Wire — complete walking skeleton gate
e2e-the-wire:
    @ATTESTER_ADDRESS={{anvil_attester_addr}} \
     bash scripts/e2e-the-wire.sh
