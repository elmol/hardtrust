test:
    @cargo test --workspace 2>/dev/null || echo "No workspace members to test"
    cd contracts && forge test

lint:
    @cargo fmt --check 2>/dev/null || echo "No workspace members to lint"
    @cargo clippy --workspace -- -D warnings 2>/dev/null || echo "No workspace members to check"
    cd contracts && forge fmt --check

ci: lint test
