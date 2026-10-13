# ADR 0002: SDK 1.0, explicit matching, and a pure Core

- Status: accepted for Phase 1 implementation.
- Date: 2026-10-13.
- Scope: the user's Phase 1 numbering, not the master document's broader phase numbering.

## Context

The user now authorizes Rust foundation implementation and requires working workspace checks/tests, stopping before HID/daemon/IPC. Existing application/driver placeholders have no source and must not become workspace members. Contributors need contracts that let a driver be registered without changing Core rules.

## Decisions

1. Implement three synchronous library crates: SDK, Registry, Core. Use stable Rust 1.99.0 (edition 2024), a generated lockfile, forbidden unsafe code, documented public APIs, and workspace linting. Do not add Tokio without asynchronous work.
2. Define SDK contract 1.0 separately from Cargo package 0.1.0 and future wire/schema versions. Typed serde DTOs reject unknown fields; version gates fail on unsupported major/newer minor. Semantic validation is explicit after parsing.
3. Separate model, physical-unit, and interface tokens. Retain observed non-UTF-8 locators and optional usage/interface/firmware; do not infer physical grouping or firmware from VID/PID.
4. Register reviewed factories explicitly, snapshot/validate their metadata, and require interface/usage evidence. All matching constraints are exact; multiple models fail closed instead of ranking an arbitrary winner.
5. Keep Core a pure in-memory inventory orchestrator. Validate complete snapshots atomically, show unknown endpoints, and reject duplicate/inconsistent identity. No discovery, connection monitor, daemon, profile database, or physical I/O in this phase.
6. Use serde for typed interchange, thiserror for typed errors, and tracing for structured library events. JSON formatting/subscription is test-only; future applications choose subscribers. Avoid logs of descriptors, serials, paths, payloads, or detailed backend errors.
7. Define bounded HID operation and driver traits without physical implementations. A checked wrapper rejects invalid requests, responses, and partial writes. Capabilities declare optional domains, but SDK 1.0 models only a volatile lighting command; declarations do not imply domain functionality.

## Consequences

All three crates can compile/test independently of a GUI, OS device libraries, or hardware. Tests can verify matching, rejection, error paths, logging, and Core behavior, but cannot establish physical compatibility. No bundled factory claims device support. Future discovery assigns identities and reviewed backends must enforce actual deadlines/rate limits; wire parsers must bound input before serde allocation.

Adding commands, serialization fields, matching behavior, or lifecycle rules needs contract/version review. Async operations, real HID access, privileged capture, and IPC require later authorized phases. See [SDK reference](../sdk.md) for actual APIs and bounds; [ADR 0001](0001-foundation-boundaries.md) remains the historical documentary foundation.
