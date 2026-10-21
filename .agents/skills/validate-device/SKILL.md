---
name: validate-device
description: Validates device manifests, driver software checks, and capability-specific hardware evidence when reviewing support for a peripheral.
compatibility: opencode
---

# Validate device support

## Inputs

Device artifacts, driver/manifest versions, sanitized protocol evidence and its handoff digest, claimed capabilities, actual check results, OS/firmware/interface matrix, and explicit physical-test authorization if requested.

## Tools

Read the binding [responsible research policy](../../../docs/reverse-engineering/responsible-research.md), [AGENTS.md](../../../AGENTS.md), [SECURITY.md](../../../SECURITY.md), the [device lifecycle](../../../docs/architecture/device-lifecycle.md) and the [agent integration](../../../docs/architecture/agent-integration.md). Inspect the real manifest validator/interpreter (`peripheral-hub-driver-declarative`, `schemas/device-manifest.schema.json`), Cargo/frontend scripts, fake transports and available laboratory equipment. The interpreter supports volatile lighting only; it is not proof of physical support.

## Procedure

1. Enumerate every support claim and its evidence. Verify identity, model versus instance, interface/usage, firmware restrictions and unsupported capabilities.
2. Validate the evidence handoff:
   ```sh
   cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- inspect --report report.json
   cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- verify --handoff local-data/handoff/ID
   ```
   A digest/assessment mismatch invalidates the handoff. `scaffold-ready` is not a validation pass; `hardware-review` still requires checking human provenance.
3. Validate manifests with the real parser:
   ```sh
   cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- validate-manifest --manifest manifest.toml
   ```
   Also test invalid versions, duplicate/missing rules, wrong interfaces, report lengths/offsets/effects, undeclared zones and unsupported operations. A valid manifest does not prove device support; a parser test does not prove runtime behavior.
4. Run actual formatting, static checks and relevant tests, recording exact commands/results. Cover ambiguous matching, wrong interface, malformed data, timeout, disconnect, permission failure and `NotSupported`. Do not report a pass for unavailable workspaces/toolchains.
5. Verify frontend/runtime labels distinguish simulation from physical operations and do not report success after transport failure. Inspect profile/schema compatibility where relevant.
6. Plan hardware tests only for known safe evidenced commands with explicit consent. Record OS, firmware, device revision, interface, driver commit, action, expected/observed result and limits. No automatic hardware CI and no unknown packet replay.
7. Assign claim labels: `simulated` for simulation, `inferred` for indirect evidence, `tested` for executed software tests, and `validated-on-hardware` only for specifically demonstrated physical capabilities. Use the separate protocol labels `confirmed-on-hardware`, `inferred` and `unverified`. Never promote a label by assertion or from an unrelated test.
8. Update `validation.md`, the capability matrix and the status ledger with actual outcomes. Hand unresolved issues to [review-contribution](../review-contribution/SKILL.md).

## Artifacts

A per-capability validation matrix, actual command results, handoff digest/verification, invalid-input evidence, limitations and a review recommendation. Documentation-only review may complete while software/hardware validation remains explicitly unavailable.

## Failure handling

If the handoff or copied report is modified, stop and regenerate from the DRA. If a manifest passes but contradicts the report, keep both facts and request review; structural validity is not a match. If hardware is unavailable or behavior is unexpected, stop and record the limitation. If tests cannot run, distinguish unavailable from passed.

## Stop conditions

Stop physical work on missing consent, unexpected behavior, uncertain safety, unavailable hardware/firmware or ambiguous identity. Stop claiming a pass when tooling is absent. Do not broaden one tested capability into full model/platform compatibility. Completion requires an honest evidence matrix for the requested scope.
