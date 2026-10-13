# ADR 0001: Documented foundation and required runtime boundaries

- Status: accepted for the documentary foundation; runtime implementation pending.
- Date: 2026-10-13.
- Basis: master specification sections 2–5 and the user's documentation-only first stage.

## Context

The repository already contains a complete organizational scaffold, but only the master specification had content at baseline. The master describes eventual software implementation; the current request restricts this stage to documentation and organization. Architecture must be explicit without pretending the software is built.

## Decisions

1. Preserve existing paths and placeholders. Fill documentation and ignore rules; do not configure fake workspaces, schemas, CI, MCP, or sample drivers.
2. Use stable Rust/Cargo and Tauri 2 + React/TypeScript/Vite with pnpm in subsequent implementation. Python is limited to justified laboratory tooling.
3. Separate the restricted desktop bridge from a user-session daemon. Use typed, versioned same-user local IPC: protected Windows named pipes and Unix sockets; no public listener or automatic elevation.
4. Support reviewed compiled Rust drivers and bounded declarative operations, not arbitrary runtime native plugins. Device matching uses interface/usage/additional evidence and distinct model/instance identities.
5. Keep Rust DRA standalone from the normal runtime. REA is optional developer tooling and MCP remains unconfigured until actual installation/startup details are verified.
6. Keep English as reference and provide priority public guides in pt-BR. Apache-2.0 remains a proposal until confirmed by the maintainer.

## Consequences

The foundation provides rules, contribution workflows, and evidence standards, but cannot launch or test the application. No package or API version, packet format, security contact, maintainer account, or hardware support is invented. Future implementations must supply real workspaces, public contracts, validators, tests, and operational commands.

Further ADRs should document concrete IPC framing/authorization, SDK versioning, declarative interpreter limits, storage migrations, and research interchange choices before those contracts are committed. This ADR records required boundaries, not experimental proof or a working design implementation.
