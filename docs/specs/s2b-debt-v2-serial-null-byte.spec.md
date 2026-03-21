# S2b-Debt-V2: Fix Trailing Null Byte in Device Serial

**Type:** Bug fix
**Depends on:** None (affects all device commands)

---

## Problem

`device capture` produces a serial with a trailing null byte (`"00000000ba807092\u0000"`) because `/sys/firmware/devicetree/base/serial-number` on Raspberry Pi includes a null terminator (FDT binary format). `String::trim()` only strips whitespace, not `\0`.

This causes on-chain verification to fail: the serial registered via `device init` → printed to terminal (null invisible) → user copies without `\0`. But `device capture` embeds the serial with `\0` into capture.json → `keccak256("serial\0")` ≠ `keccak256("serial")` → UNVERIFIED.

The fix already exists in the codebase: `device/src/lib.rs:170` uses `.trim_end_matches('\0')` for the `model` field from the same device tree source. It was never applied to `read_serial()`.

## What to Build

### Step 1 — Fix `read_serial()` in `device/src/main.rs`

Change line ~60 from:

```rust
.map(|s| s.trim().to_string())
```

to:

```rust
.map(|s| s.trim().trim_end_matches('\0').to_string())
```

### Step 2 — Apply same fix to any other device tree reads

Audit all reads from `/sys/firmware/devicetree/`, `/proc/device-tree/` for the same issue. The `camera_info` field also reads from `/proc/device-tree/model` — verify it already has the fix.

---

## Files

| File | Action |
|------|--------|
| `device/src/main.rs` | Add `.trim_end_matches('\0')` to `read_serial()` |

## Tests (TDD — write first)

1. **Serial without null byte** — `read_serial()` on a value with no `\0` returns unchanged
2. **Serial with trailing null byte** — `read_serial()` on `"00000000ba807092\0"` returns `"00000000ba807092"`
3. **Capture serial matches init serial** — both produce identical serial (no null mismatch)
4. **E2E Cases 1-6 still pass**

## Validation Criteria

- [ ] `device init` and `device capture` produce identical serial strings
- [ ] capture.json serial has no trailing `\0` or `\u0000`
- [ ] On-chain verification succeeds for captures from RPi devices
- [ ] `just ci` passes
