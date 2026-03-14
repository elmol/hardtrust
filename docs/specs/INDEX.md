# Specs Index

Traceability map between user stories, specs, and handoff prompts.

## Setup & Infrastructure

| Spec ID | Name | Story | Status | Handoff |
|---------|------|-------|--------|---------|
| S0.1 | [Repo Initialization](s0.1-repo-initialization.spec.md) | — | Implemented | [s0.1-handoff](../handoff/s0.1-handoff.md) |
| S0.2 | [CI Pipeline](s0.2-ci-pipeline.spec.md) | — | Implemented | [s0.2-handoff](../handoff/s0.2-handoff.md) |
| S0.3 | [AI Workflow Setup](s0.3-ai-workflow-setup.spec.md) | — | Implemented | [s0.3-handoff](../handoff/s0.3-handoff.md) |
| S0.4 | [Seed ADRs](s0.4-seed-adrs.spec.md) | — | Implemented | [s0.4-handoff](../handoff/s0.4-handoff.md) |
| S0.5 | [AI Dev Practices](s0.5-ai-dev-practices.spec.md) | — | Implemented | [s0.5-handoff](../handoff/s0.5-handoff.md) |
| S0.6 | [CLAUDE.md Updates](s0.6-hardtrust-claude-md-updates.spec.md) | — | Implemented | [s0.6-handoff](../handoff/s0.6-handoff.md) |
| S0.7 | [Solidity Static Analysis](s0.7-solidity-static-analysis.spec.md) | — | Implemented | [s0.7-handoff](../handoff/s0.7-handoff.md) |

## Slice 1 — CLI/Console

### S1a — "The Wire" (hardcoded end-to-end)

Orchestration handoff: [s1a-handoff](../handoff/s1a-handoff.md)

#### S1a.1 — Register device on-chain

**Story:** [S1a.1](../stories/slice-1/s1a.1-register-device-on-chain.md)

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| V1 | [Registry Contract](s1a.1-v1-registry-contract.spec.md) | Implemented | Embedded |
| V2 | [Device CLI](s1a.1-v2-device-cli.spec.md) | Review | Embedded |
| V3 | [Attester CLI](s1a.1-v3-attester-cli.spec.md) | Review | Embedded |
| V4 | [E2E Validation](s1a.1-v4-e2e-register.spec.md) | Review | Embedded |

V1 and V2 can run in parallel. V3 depends on V1. V4 depends on all.

#### S1a.2 — Verify registered device

**Story:** [S1a.2](../stories/slice-1/s1a.2-verify-registered-device.md) | Specs: TBD

#### S1a.3 — Verify unregistered device

**Story:** [S1a.3](../stories/slice-1/s1a.3-verify-unregistered-device.md) | Specs: TBD

### S1b — "Real Crypto" | Pending

### S1c — "Polish & CI" | Pending

## Notes

- **Setup specs (S0.x)** have no user story — infrastructure prerequisites
- **Story specs (V1, V2...)** are tiny vertical specs implementing one concern each
- **Status:** Draft → Review → Approved → Implemented
- **Execution order:** S0.x → S1a.1 (V1+V2 parallel, then V3, then V4) → S1a.2 → S1a.3 → S1b → S1c
