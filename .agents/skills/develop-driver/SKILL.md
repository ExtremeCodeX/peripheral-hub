---
name: develop-driver
description: Develops evidence-backed declarative or compiled Rust device drivers when implementation is authorized and Peripheral Hub SDK contracts are available.
compatibility: opencode
---

# Develop a driver

## Inputs

Authorized implementation scope, sanitized protocol evidence, identity/interface constraints, capability limits, known firmware revisions, and implemented SDK/transport/registry/schema contracts.

## Tools

Read the binding [responsible research policy](../../../docs/reverse-engineering/responsible-research.md), [AGENTS.md](../../../AGENTS.md), [SDK 1.0](../../../docs/architecture/sdk.md), [driver architecture](../../../docs/architecture/driver-system.md), [HID/daemon guide](../../../docs/architecture/daemon.md), and [protocol format](../../../docs/protocols/specification-format.md). Implement drivers independently from permissible protocol facts; never reproduce proprietary/decompiled source or bypass protections. `HidDiscovery::open_reviewed`, `HidPolicy` and checked report transport exist, but no physical factory/policy is bundled. Root check/tests are real; daemon `--simulate` is an explicit fictitious example only. Declarative interpreter, manifest validators and scaffold utilities remain absent.

## Procedure

1. Confirm scope and evidence. Current Phase 2 implements transport and an explicit simulator, not proprietary hardware support or manifest execution. A physical driver needs reviewed protocol/policy, authorization and hardware validation. Do not create source-less manifests or invent absent validator/interpreter APIs.
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
