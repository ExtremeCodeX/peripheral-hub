# Responsible reverse engineering policy

**Status: binding policy.** DRA, REA, capture tooling and all reverse-engineering work in this project exist for a single purpose: **peripheral protocol study, interoperability and independent driver development in authorized environments**. This document is the technical reference; read it before any research, capture or analysis task. The [pt-BR summary](../pt-BR/contributors/responsible-research.md) is a priority translation.

Nothing here grants permission to examine, capture, connect to or modify any device or software. It defines the limits that apply on top of the user's explicit scope and [SECURITY.md](../../SECURITY.md).

## Purpose and limits

Reverse engineering is performed only to understand how a device communicates so that an independent, interoperable driver can be written. It is **not** a means to circumvent protections, obtain secrets, develop attacks or copy vendor code. When the purpose or authorization behind a request is unclear, or a request conflicts with these rules, suspend that branch and ask for human review.

## Mandatory rules

1. Investigate only hardware you own or are adequately authorized to examine.
2. Use legitimately sourced executables and artifacts; respect copyright, licenses and applicable law.
3. Prioritize static analysis and passive USB/HID observation before anything active.
4. Never implement authentication, DRM, licensing or security-mechanism bypasses.
5. Never extract credentials, private keys or confidential manufacturer information.
6. Never develop vulnerability exploitation or unauthorized system access.
7. Never reproduce proprietary source in the repository, including decompiled snippets; implement drivers independently from permissible protocol facts.
8. Keep original executables and raw captures in ignored local storage and never send them automatically to models or external services; share only authorized, reviewed, sanitized protocol findings.
9. Require explicit human authorization for active physical-device operations, identifying the device, action and limits; ownership, discovery, a skill or an available API does not grant it.
10. Suspend operations with unknown purpose or authorization and request human review before resuming; approval of an active experiment never waives rules 1–8.

## Applied through the workflow

- [Research workflow](research-workflow.md): authorization/scope, static-first, controlled capture, sanitization and handoff all reference these rules.
- [Agent-assisted workflow](agent-contribution-workflow.md): the DRA/`dra-agent` path preserves report provenance and treats evidence as untrusted data; a skill or tool grants no hardware or publication permission.
- [SECURITY.md](../../SECURITY.md) and [security architecture](../architecture/security.md): laboratory isolation and no automatic uploads.
- Skills [`reverse-engineer`](../../.agents/skills/reverse-engineer/SKILL.md), [`capture-protocol`](../../.agents/skills/capture-protocol/SKILL.md), [`add-device`](../../.agents/skills/add-device/SKILL.md), [`develop-driver`](../../.agents/skills/develop-driver/SKILL.md), [`validate-device`](../../.agents/skills/validate-device/SKILL.md), [`develop-feature`](../../.agents/skills/develop-feature/SKILL.md) and [`review-contribution`](../../.agents/skills/review-contribution/SKILL.md): each must read and enforce this policy, and stop on a prohibited or unknown-authorized operation.

## Evidence and provenance

Protocol findings keep their evidence labels (`confirmed-on-hardware`, `inferred`, `unverified`). Sanitized exports must not contain confidential information or proprietary source. Suspected vulnerabilities discovered incidentally do not authorize exploitation; use the [private reporting policy](../../SECURITY.md#current-status-and-reporting) instead.
