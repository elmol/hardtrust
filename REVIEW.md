# REVIEW.md

## Spec and Story Compliance
- Does the implementation satisfy all acceptance criteria from the user story?
- Does the implementation match the spec's "What to Build" section?
- Does the implementation avoid everything in the spec's "What NOT to Build" section?
- Are all edge cases from the story handled?

## Code Quality (Rust)
- No `.unwrap()` in library code (`common/`) — use `Result` with descriptive errors
- `.unwrap()` is acceptable in tests and in top-level CLI error handling (with `.expect("message")`)
- All public functions have doc comments
- Error messages are actionable — they tell the user what went wrong and what to do

## Code Quality (Solidity)
- No hardcoded private keys or secrets
- Access control on state-changing functions
- Events emitted for all state changes (deferred in S1a walking skeleton — added in S1b)
- NatSpec comments on public/external functions

## Security
- No secrets (private keys, mnemonics, API keys) in code or committed files
- File permissions set correctly for key material (0o600 for files, 0o700 for directories)
- Input validation on all external inputs (CLI args, contract function parameters)

## Testing
- Tests exist for new functionality
- Each acceptance criterion has at least one corresponding test
- Edge cases from the story are covered
- Tests run in CI (emulation mode where needed)

## Commits and PR
- Conventional commit messages on all commits
- PR description references the story ID and spec ID
- Changes are scoped to the story — no unrelated modifications
