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
| S0.8 | [CI End-to-End Tests](s0.8-ci-e2e.spec.md) | — | Implemented | Embedded |

## Slice 1 — CLI/Console

### S1a — "The Wire" (hardcoded end-to-end)

Orchestration handoff: [s1a-handoff](../handoff/s1a-handoff.md)

#### S1a.1 — Register device on-chain

**Story:** [S1a.1](../stories/slice-1/s1a.1-register-device-on-chain.md)

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| V1 | [Registry Contract](s1a.1-v1-registry-contract.spec.md) | Implemented | Embedded |
| V2 | [Device CLI](s1a.1-v2-device-cli.spec.md) | Implemented | Embedded |
| V3 | [Attester CLI](s1a.1-v3-attester-cli.spec.md) | Implemented | Embedded |
| V4 | [E2E Validation](s1a.1-v4-e2e-register.spec.md) | Implemented | Embedded |

V1 and V2 can run in parallel. V3 depends on V1. V4 depends on all.

#### S1a.2 — Verify registered device

**Story:** [S1a.2](../stories/slice-1/s1a.2-verify-registered-device.md)

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| V1 | [Types & Device Emit](s1a.2-v1-types-and-device-emit.spec.md) | Implemented | Embedded |
| V2 | [Attester Verify](s1a.2-v2-attester-verify.spec.md) | Implemented | Embedded |
| V3 | [E2E Verify](s1a.2-v3-e2e-verify.spec.md) | Implemented | Embedded |

V1 first (types + device emit). V2 depends on V1. V3 depends on V1 + V2.

#### S1a.3 — Verify unregistered device

**Story:** [S1a.3](../stories/slice-1/s1a.3-verify-unregistered-device.md)

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| V1 | [E2E The Wire](s1a.3-v1-e2e-the-wire.spec.md) | Implemented | Embedded |

This is the FINAL gate of "The Wire" walking skeleton. V1 depends on all S1a.1 and S1a.2 specs.

#### S1a Refactor — Polish The Wire

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| S1a-R | [Refactor The Wire](s1a-refactor.spec.md) | Implemented | Embedded |

### S1b — "Real Crypto"

#### S1b.1 — Real Keys

**Story:** [S1b.1](../stories/slice-1/s1b.1-real-keys.md)

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| V1 | [Key Generation](s1b.1-v1-key-generation.spec.md) | Implemented | Embedded |
| V2 | [Device Init](s1b.1-v2-device-init.spec.md) | Implemented | Embedded |

V1 first (pure function in types/). V2 depends on V1.

#### S1b.2 — Signed Reading

**Story:** [S1b.2](../stories/slice-1/s1b.2-signed-reading.md)

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| V1 | [sign_reading function](s1b.2-v1-sign-reading.spec.md) | Implemented | Embedded |
| V2 | [Device Emit](s1b.2-v2-device-emit.spec.md) | Implemented | Embedded |

V1 depends on S1b.1-V1 (k256 in types/). V2 depends on V1 + S1b.1-V2.

#### S1b.3 — Real Verify

**Story:** [S1b.3](../stories/slice-1/s1b.3-real-verify.md)

| Spec | Name | Status | Handoff |
|------|------|--------|---------|
| V1 | [verify_reading function](s1b.3-v1-verify-reading.spec.md) | Implemented | Embedded |
| V2 | [Attester Verify](s1b.3-v2-attester-verify.spec.md) | Implemented | Embedded |

V1 depends on S1b.2-V1 (sign_reading, for round-trip test). V2 depends on V1.

### S1c — "Polish & CI" | Pending

## Notes

- **Setup specs (S0.x)** have no user story — infrastructure prerequisites
- **Story specs (V1, V2...)** are tiny vertical specs implementing one concern each
- **Status:** Draft → Review → Approved → Implemented
- **Execution order:** S0.x → S1a.1 (V1+V2 parallel, then V3, then V4) → S1a.2 → S1a.3 → S1b → S1c
