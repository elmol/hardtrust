# CLAUDE.md

## Operational Rules

- Read the spec at `docs/specs/[spec-id].spec.md` before writing any code
- Run `just ci` before creating a PR
- One story per branch, one PR per story
- Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `test:`
- Create an ADR in `docs/adr/` when choosing between viable alternatives or deviating from architecture
- Never merge autonomously — always require human approval
- When CI fails, diagnose and fix before requesting review
- Use `--emulate` flag or `HARDTRUST_EMULATE=1` for development/CI without RPi hardware
- No direct pushes to main — use PRs

## Branch Protection (configured via GitHub UI)

- Require CI pass before merge
- Require 1 approval
- No direct push to `main`
