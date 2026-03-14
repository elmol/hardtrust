# REVIEW.md

Review rules for automated and human code review:

- Check spec compliance — does the implementation match the spec?
- No `.unwrap()` in library code (use proper error handling)
- No hardcoded private keys or secrets
- All public functions must have doc comments
- Tests must exist for new functionality
- Conventional commit messages on all commits
