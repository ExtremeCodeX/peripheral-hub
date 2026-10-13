---
name: develop-driver
description: Develops evidence-backed declarative or compiled Rust device drivers when implementation is authorized and Peripheral Hub SDK contracts are available.
compatibility: opencode
---

# Develop a driver

## Inputs

Authorized implementation scope, sanitized protocol evidence, identity/interface constraints, capability limits, known firmware revisions, and implemented SDK/transport/registry/schema contracts.

## Tools

Read [AGENTS.md](../../../AGENTS.md), [driver architecture](../../../docs/architecture/driver-system.md), and the [protocol format](../../../docs/protocols/specification-format.md). Inspect actual Cargo members and validators. Rust/Cargo, fake transport, schema/semantic checks, and scaffold tools are conditional on implementation; none is currently an available project command.

## Procedure

1. Confirm implementation authorization and SDK readiness. In the foundation stage, document the design and blockers only. Do not create a Cargo manifest without source or invent a compatible SDK.
2. Choose declarative support only for safe operations already modeled by the bounded interpreter. Use compiled, explicitly registered Rust for complex framing, sequencing, or state. Never load arbitrary runtime DLLs.
3. Specify identity using VID/PID plus interfaces, usage, revision evidence, and ambiguity handling. Separate model ID from physical instance ID.
4. Implement only evidenced capabilities. Define byte/type/range limits, timeouts, rate limits, lifecycle, ACK/response handling, and explicit errors. Unknown or absent operations return `NotSupported`; transport failure must not return success.
5. Validate manifest versions, typed TOML, interchange schema, and semantic constraints. Treat external manifests as untrusted. If safe execution cannot be guaranteed, keep declarative behavior preview-only.
6. Add meaningful fake-transport tests for exact evidenced framing, limits, wrong interfaces, ambiguity, malformed responses, disconnects, timeouts, and unsupported operations. Synthetic fixtures demonstrate code behavior only.
7. Run available formatting/static checks/tests in the repository's order, recording exact results. Hardware experiments belong to separately authorized [validate-device](../validate-device/SKILL.md) work.
8. Document protocol provenance, uncertainty, validation matrix, registration/build changes, and public guides. Prepare a review proposal with only sanitized authorized artifacts.

## Artifacts

An actual implemented driver when authorized, typed/validated manifest, `protocol.md`, `validation.md`, software tests, sanitized fixtures, registry integration, and recorded check results. Fictitious demonstration drivers belong in `examples/`, clearly labeled.

## Stop conditions

Stop on missing contracts, contradictory evidence, ambiguous identity, unbounded declarative behavior, unknown writes, or unapproved physical execution. No firmware/bootloader/persistent-memory operations. Completion requires actual software checks for implemented scope and honest separate hardware evidence, not a simulated claim of physical success.
