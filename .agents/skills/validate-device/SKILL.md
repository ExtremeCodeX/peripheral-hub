---
name: validate-device
description: Validates device manifests, driver software checks, and capability-specific hardware evidence when reviewing support for a peripheral.
compatibility: opencode
---

# Validate device support

## Inputs

Device artifacts, driver/manifest versions, protocol evidence, claimed capabilities, check results, OS/firmware/interface matrix, and explicit physical-test authorization if requested.

## Tools

Read the binding [responsible research policy](../../../docs/reverse-engineering/responsible-research.md), [AGENTS.md](../../../AGENTS.md), [SECURITY.md](../../../SECURITY.md), and the [device lifecycle](../../../docs/architecture/device-lifecycle.md). Inspect the implemented manifest validator/interpreter (`peripheral-hub-driver-declarative`, `schemas/device-manifest.schema.json`), Cargo/frontend scripts, fake transports, and available laboratory equipment. The interpreter supports volatile lighting only; it is not evidence of physical support.

## Procedure

1. Enumerate every support claim and its evidence. Verify identity, model versus instance, interface/usage, revision restrictions, and unsupported capabilities.
2. Validate manifests with the implemented schema/semantic validator: check valid and invalid inputs, versions, limits, offsets/effects, undeclared zones and unsupported operations. Invoke the interpreter only for modeled volatile-lighting operations; record any unsupported path as unavailable, not passed.
3. Run actual formatting, static checks, and relevant unit/integration tests, including ambiguous matching, wrong interfaces, malformed data, timeout, disconnect, permission failure, and `NotSupported`. Record commands and outputs precisely.
4. Verify frontend/runtime labels distinguish simulation from physical operations and do not report success after a daemon/transport failure. Inspect profile/schema compatibility where relevant.
5. Plan opt-in hardware tests only for known safe evidenced commands and explicit consent. Record OS, firmware, device revision, interface, driver commit, action, expected result, observed result, and limitations. No automatic hardware CI.
6. Assign claim labels: `simulated` for simulation, `inferred` for indirect evidence, `tested` for executed software tests, and `validated-on-hardware` only for specifically demonstrated physical capabilities. A passing test does not promote unrelated capabilities or firmware revisions.
7. Use separate hypothesis labels (`confirmed-on-hardware`, `inferred`, `unverified`) in the protocol report. Document unsupported/untested combinations and observed failures.
8. Update `validation.md`, capability matrix, and status ledger; hand unresolved issues to [review-contribution](../review-contribution/SKILL.md).

## Artifacts

A per-capability validation matrix, actual command results, safe hardware records if authorized, invalid-input evidence, limitations, and a review recommendation. Documentation-only review may complete while software/hardware validation remains unavailable.

## Stop conditions

Stop physical work on missing consent, unexpected behavior, uncertain safety, unavailable hardware/firmware, or ambiguous identity. Stop claiming passed software validation when workspaces/tools are absent. Do not broaden one tested capability into full model/platform compatibility. Completion requires an honest evidence matrix for the requested scope, with gaps explicitly marked.
