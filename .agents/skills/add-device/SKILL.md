---
name: add-device
description: Coordinates identification, protocol evidence, driver development, validation, and review when adding a peripheral to Peripheral Hub.
compatibility: opencode
---

# Add a device

## Inputs

- Requested model and feature scope; authorization to examine the hardware/software.
- Observed OS, transport, firmware/revision, interfaces, VID/PID and usage values where available.
- Existing device entries, safe research records, and the current implementation status.

## Tools

Use repository inspection and guides. [SDK 1.0](../../../docs/architecture/sdk.md), Registry and Core are preserved; [Phase 2](../../../docs/architecture/daemon.md) supplies real non-destructive `cargo run -p peripheral-hub-daemon -- enumerate` and explicit `serve --simulate` diagnostics. DRA, proprietary physical drivers, manifest validators and scaffold tools remain absent. REA/capture tools require verified installation/authorization. Read the binding [responsible research policy](../../../docs/reverse-engineering/responsible-research.md), [AGENTS.md](../../../AGENTS.md), [SECURITY.md](../../../SECURITY.md), and [device guide](../../../docs/en/contributors/adding-a-device.md).

## Procedure

1. Inspect the master specification, current scope, existing directories, and matching device documentation. Do not duplicate an entry or treat a structural placeholder as a driver.
2. Record model versus physical instance identity and all relevant interface/usage evidence. Resolve ambiguous matches before proposing support.
3. Inventory evidence and missing facts. Use [reverse-engineer](../reverse-engineer/SKILL.md) for authorized static observations and [capture-protocol](../capture-protocol/SKILL.md) for consented action/capture correlation. Keep raw data ignored and local.
4. Produce a capability-by-capability protocol record with provenance, safe limits, and uncertainty. An installer or VID/PID alone is insufficient proof.
5. If implementation is authorized and SDK/validators exist, use [develop-driver](../develop-driver/SKILL.md); otherwise document the prerequisite and stop that branch without creating fictitious code or packets.
6. Use [validate-device](../validate-device/SKILL.md) to distinguish software tests from optional authorized physical validation. Keep firmware/platform gaps visible.
7. Update public documentation and the status ledger. Use [review-contribution](../review-contribution/SKILL.md) to prepare a human-reviewed PR proposal; publishing requires explicit authorization.

## Artifacts

A sanitized identity/evidence summary, capability matrix, unresolved questions, and eventually `devices/<manufacturer>/<model>/{manifest.toml,protocol.md,validation.md,tests/,fixtures/}`. Native drivers add source only when implemented/evidence-backed. Current Phase 2 provides communication infrastructure and simulation; enumeration alone does not authorize unknown operations or establish model support.

## Stop conditions

Stop affected work on absent consent, uncertain identity, unknown command safety, missing SDK/tooling, or insufficient evidence. No raw binary redistribution, unknown replay, firmware/persistent writes, fabricated validation, or automatic PR publication. Completion means evidence and review requirements for the actual scope are met; directory creation alone is not support.
