---
name: develop-feature
description: Guides application feature changes across Rust crates, daemon, IPC, and Tauri React layers when software implementation is explicitly in scope.
compatibility: opencode
---

# Develop an application feature

## Inputs

User-authorized scope, acceptance criteria, current implementation status, affected layers/contracts, supported platforms, and persistence/security implications.

## Tools

Read [AGENTS.md](../../../AGENTS.md), [architecture overview](../../../docs/architecture/overview.md), [SDK reference](../../../docs/architecture/sdk.md), [IPC design](../../../docs/architecture/ipc.md), and existing ADRs. Phase 1 has three real Cargo packages: `peripheral-hub-device-sdk`, `peripheral-hub-device-registry`, `peripheral-hub-core`. Run root `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --workspace`, and `cargo test --workspace`. Application/pnpm/IPC manifests remain placeholders; later layers need separate authorization.

## Procedure

1. Define observable acceptance criteria, owning layer, dependencies, and existing behavior to preserve. If scope is documentation-only, produce documentation without application code or dummy workspaces.
2. Keep React → restricted Tauri API → versioned typed IPC → user-session daemon → SDK/drivers → transport boundaries. DRA stays standalone; REA remains optional tooling.
3. Document significant decisions with an ADR. Define explicit DTO/error/version contracts before integration, including unsupported operations, disconnects, permission failures, and timeout behavior.
4. Implement a narrow real increment using established project conventions. Use stable Rust and strict TypeScript; avoid extra frameworks, Python runtime, privileged daemons, arbitrary plugins, and public network listeners.
5. Define typed/semantic input validation, size/rate limits, SQLite migrations, and versioned profiles where needed. Do not imply a requested hardware operation succeeded without transport evidence.
6. Provide loading, empty, disconnected, denied, error, simulated, and unsupported frontend states with accessible interactions. Initial macros cover editing/validation/persistence/simulation only.
7. Add meaningful contract/error-path tests, then run available checks in the repository's prescribed order. Use only frontend scripts actually defined. Keep hardware tests separately authorized and opt-in.
8. Update English reference documentation, priority pt-BR guides, ADRs, and the status ledger with actual outcomes and blockers. Submit the scoped change for human review when authorized.

## Artifacts

Implemented code/tests within the authorized phase, public contract documentation, relevant ADRs, translation updates, check results, and a precise status update. Documentation-only requests still produce no application code.

## Stop conditions

Stop the affected branch on absent implementation permission, missing workspaces/contracts, privacy exposure, unsafe hardware operations, or a boundary violation requiring redesign. Do not invent successful builds or application commands. Completion requires acceptance criteria verified by real available checks, with limitations distinguished from delivered behavior.
