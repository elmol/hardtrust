# CLAUDE.md

## Context Loading

Before implementing any story, read these documents in order:

1. **Story:** `docs/stories/slice-N/[story-id]-[name].md` — acceptance criteria, edge cases, dependencies
2. **Spec:** `docs/specs/[spec-id].spec.md` — what to build, what NOT to build, validation criteria
3. **Architecture:** `docs/architecture.md` — system boundaries, component responsibilities, data flow
4. **ADRs:** scan `docs/adr/` for any ADR relevant to the component you are modifying
5. **REVIEW.md** — review criteria that your PR will be checked against

For non-functional (S0.x) specs that have no story, skip step 1.

When a spec references other specs or stories (e.g., "depends on S1.1"), read those too.

## Operational Rules

### Planning
- Read context documents (see Context Loading above) before writing any code
- Break the story into sub-tasks if it touches multiple crates
- Identify which crate(s) and files will be created or modified

### Implementation
- One story per branch, one PR per story
- Branch naming: `feat/[story-id]-short-description` (e.g., `feat/s1.1-device-init`), or `fix/[story-id]-description`, `chore/description`
- Commit granularly — one commit per logical change, not one giant commit per story
- Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`
- When pushing a branch, always create the PR in the same step — do not wait for a separate request
- Run `just ci` before creating a PR
- Use `--emulate` flag or `HARDTRUST_EMULATE=1` for development/CI without RPi hardware

### Quality
- Tests are part of the story — write tests alongside implementation, not after
- Every acceptance criterion in the story should have at least one test
- No `.unwrap()` in library code (`common/`) — use `Result` with descriptive errors
- All public functions must have doc comments

### Architecture Decisions
- Create an ADR in `docs/adr/` when choosing between viable alternatives or deviating from architecture
- ADRs are committed alongside the code change that implements the decision

### Boundaries
- Never merge autonomously — always require human approval
- No direct pushes to main — use PRs
- Do not implement anything listed in the spec's "What NOT to Build" section

## When Things Fail

### CI failure
1. Read the full CI output
2. Identify the failing gate (lint, test, integration, etc.)
3. Fix the issue locally
4. Run `just ci` to verify the fix
5. Commit the fix with `fix: [description of what broke]`
6. If stuck after 3 attempts, report the failure with diagnosis to the human

### Test failure
1. Read the test output and identify the failing assertion
2. Determine if the bug is in the code or the test
3. If in the code: fix and verify
4. If in the test: check the spec/story — does the test match the acceptance criteria?
5. Run the full test suite to check for regressions

### Review feedback
1. Read all review comments
2. Fix each issue in a separate commit (or group related fixes)
3. Do not force-push — add fixup commits so the reviewer can see what changed
4. Run `just ci` after all fixes

## Documentation Cross-References

- Specs reference their source story via a "Story Reference" link at the top
- Specs and stories should reference relevant ADRs (e.g., "See ADR-0002 for why secp256k1")
- Before implementing, read any ADR referenced by the spec or story — it contains the rationale behind technical decisions
- When implementing, always verify your code satisfies BOTH the spec's validation criteria AND the story's acceptance criteria
- If a spec and story contradict each other, the spec takes precedence (specs are more recent and more detailed)
- If an implementation decision contradicts an existing ADR, do not proceed — report it to the human
- Report any contradictions to the human rather than guessing

## Pre-Commit Validation

Before EVERY `git commit`, run `just ci`. If it fails:

1. Run `cargo fmt` and `forge fmt` to auto-fix formatting
2. Fix any clippy or solhint warnings in the code
3. Run `just ci` again to confirm all checks pass
4. Only then proceed with the commit

Never commit with known lint or test failures. If `just ci` fails 3 times after attempting fixes, stop and report the issue to the human.

## Build Order

Foundry contracts must be built before Rust crates that depend on ABI output:

1. `cd contracts && forge build` — generates `contracts/out/`
2. `cargo build --workspace` — Alloy `sol!` macro reads from `contracts/out/`

If you see "file not found" errors from `sol!`, run `forge build` first.

## Local Development (Anvil)

The walking skeleton uses Anvil (Foundry's local chain) with deterministic accounts:

| Account | Role     | Address                                    | Private Key                                                        |
|---------|----------|--------------------------------------------|--------------------------------------------------------------------|
| #0      | Deployer | 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 | 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 |
| #1      | Attester | 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 | 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d |

These are well-known Anvil test keys — NOT secrets. Do not flag them as security issues.

## Available Skills

Install skills as needed — do not preload skills for code that does not exist yet.

| Skill | Install When | Purpose |
|-------|-------------|---------|
| `trailofbits/building-secure-contracts` | First Solidity PR (S1.2+) | Smart contract vulnerability scanning |
| `trailofbits/static-analysis` | When codebase is large enough for SARIF analysis | Multi-tool static analysis |

## MCP Servers

No MCP servers configured. All Slice 1 development uses local tools only (Foundry, Cargo, just).

## Branch Protection (configured via GitHub UI)

- Require CI pass before merge
- Require 1 approval
- No direct push to `main`
