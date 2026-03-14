# CLAUDE.md

- Read specs from `docs/specs/` before implementing any story
- Create an ADR in `docs/adr/` when making a significant technical decision
- Use conventional commits: feat:, fix:, chore:, docs:, test:
- Run `just ci` before pushing
- No direct pushes to main — use PRs

## Branch Protection (configured via GitHub UI)

- Require CI pass before merge
- Require 1 approval
- No direct push to `main`
