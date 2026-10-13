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

Use repository inspection and the contributor guides. [SDK 1.0](../../../docs/architecture/sdk.md), Registry and Core are implemented and checked with `cargo check --workspace` / `cargo test --workspace`. DRA, physical backends, manifest validators and scaffold tools are unimplemented. Optional REA and USBPcap/Wireshark/TShark require verified installation and appropriate authorization. Read [AGENTS.md](../../../AGENTS.md), [SECURITY.md](../../../SECURITY.md), and the [device guide](../../../docs/en/contributors/adding-a-device.md).

## Procedure

1. Inspect the master specification, current scope, existing directories, and matching device documentation. Do not duplicate an entry or treat a structural placeholder as a driver.
2. Record model versus physical instance identity and all relevant interface/usage evidence. Resolve ambiguous matches before proposing support.
3. Inventory evidence and missing facts. Use [reverse-engineer](../reverse-engineer/SKILL.md) for authorized static observations and [capture-protocol](../capture-protocol/SKILL.md) for consented action/capture correlation. Keep raw data ignored and local.
4. Produce a capability-by-capability protocol record with provenance, safe limits, and uncertainty. An installer or VID/PID alone is insufficient proof.
5. If implementation is authorized and SDK/validators exist, use [develop-driver](../develop-driver/SKILL.md); otherwise document the prerequisite and stop that branch without creating fictitious code or packets.
6. Use [validate-device](../validate-device/SKILL.md) to distinguish software tests from optional authorized physical validation. Keep firmware/platform gaps visible.
7. Update public documentation and the status ledger. Use [review-contribution](../review-contribution/SKILL.md) to prepare a human-reviewed PR proposal; publishing requires explicit authorization.

## Artifacts

A sanitized identity/evidence summary, capability matrix, unresolved questions, and eventually `devices/<manufacturer>/<model>/{manifest.toml,protocol.md,validation.md,tests/,fixtures/}`. Native drivers add Cargo/source files only when implemented. Current Phase 1 does not authorize adding physical drivers; report that prerequisite rather than claiming support.

## Stop conditions

Stop affected work on absent consent, uncertain identity, unknown command safety, missing SDK/tooling, or insufficient evidence. No raw binary redistribution, unknown replay, firmware/persistent writes, fabricated validation, or automatic PR publication. Completion means evidence and review requirements for the actual scope are met; directory creation alone is not support.
