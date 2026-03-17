# S1-Debt-V4: Dev Config Extraction

**Type:** Refactor (tech debt)
**Story Reference:** N/A — tech debt cleanup before Slice 2
**Depends on:** S1-Debt-V3 (dev_config.rs changes)

---

## What to Build

Replace hardcoded `DEV_PRIVATE_KEY` and `DEV_RPC_URL` with environment variables, so the attester binary is safe to run against any network.

### 1. Update `attester/src/main.rs`

Replace direct imports of `dev_config::DEV_PRIVATE_KEY` and `dev_config::DEV_RPC_URL` with environment variable reads:

```rust
let rpc_url = std::env::var("HARDTRUST_RPC_URL")
    .unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());

let private_key = std::env::var("HARDTRUST_PRIVATE_KEY")
    .map_err(|_| "HARDTRUST_PRIVATE_KEY env var is required")?;
```

- `HARDTRUST_RPC_URL`: Optional, defaults to localhost Anvil
- `HARDTRUST_PRIVATE_KEY`: **Required** — no default. Forces explicit configuration.
- `HARDTRUST_CONTRACT_ADDRESS`: Keep existing pattern (already read from CLI args or config)

### 2. Update `protocol/src/dev_config.rs`

After S1-Debt-V3 removes `DEV_SERIAL` and `DEV_ADDRESS`, this file should only contain `DEV_CONTRACT_ADDRESS` (if still needed by e2e). If nothing remains that Rust code uses, gate the entire module with `#[cfg(test)]` or remove it.

### 3. Update `scripts/e2e-the-wire.sh`

Export `HARDTRUST_PRIVATE_KEY` and `HARDTRUST_RPC_URL` before calling the attester binary:

```bash
export HARDTRUST_RPC_URL="http://127.0.0.1:8545"
export HARDTRUST_PRIVATE_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
```

### 4. Update `README.md` usage section

Document the new env vars:

```
## Configuration

| Env Var | Required | Default | Description |
|---------|----------|---------|-------------|
| HARDTRUST_PRIVATE_KEY | Yes | — | Attester signing key (hex) |
| HARDTRUST_RPC_URL | No | http://127.0.0.1:8545 | RPC endpoint |
```

---

## Files touched

- `attester/src/main.rs` (env var reads instead of dev_config imports)
- `protocol/src/dev_config.rs` (gate with `#[cfg(test)]` or remove)
- `scripts/e2e-the-wire.sh` (export env vars)
- `README.md` (document env vars)

## What NOT to Build

- Do not add CLI flags (--rpc-url, --private-key) — env vars are sufficient for v1
- Do not add .env file support or dotenv dependency
- Do not change the device binary (it doesn't use these configs)

## TDD Order

1. **Test:** Verify `just e2e-the-wire` works before changes (baseline)
2. **Implement:** Apply env var changes
3. **Test:** `just e2e-the-wire` passes (script exports vars)
4. **Test:** Running attester WITHOUT env vars shows clear error message
5. **Validate:** `just ci` passes

## Validation Criteria

- [ ] No hardcoded private key in non-test Rust code
- [ ] `HARDTRUST_PRIVATE_KEY` is required — attester fails with clear message if unset
- [ ] `HARDTRUST_RPC_URL` defaults to localhost if unset
- [ ] `just e2e-the-wire` passes (script sets env vars)
- [ ] `just ci` passes
- [ ] README documents env vars
