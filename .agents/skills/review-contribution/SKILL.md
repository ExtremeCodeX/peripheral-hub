---
name: review-contribution
description: Reviews documentation, feature, or driver changes for architectural safety, evidence, compatibility, licensing, and truthful validation before human acceptance.
compatibility: opencode
---

# Review a contribution

## Inputs

Requested review scope, actual change set/base when Git is available, contribution rationale, affected contracts, check results, sanitized evidence, and known limitations.

## Tools

Read [CONTRIBUTING.md](../../../CONTRIBUTING.md), [SECURITY.md](../../../SECURITY.md), [AGENTS.md](../../../AGENTS.md), and the [status ledger](../../../docs/architecture/implementation-status.md). Inspect files/diffs, real tests/validators, and artifact provenance. Use Git only if initialized; no Git metadata currently exists.

## Procedure

1. Identify actual changes and pre-existing work. Compare requested scope with delivery; flag software introduced during documentation-only work and placeholders represented as implementations.
2. Review architecture: restricted frontend, same-user versioned IPC, least privilege, compiled reviewed native drivers, bounded declarative execution, and distinct model/instance identity.
3. Inspect parsers, command limits, unsupported errors, logs, transport failures, migrations, dependency changes, and fixture sanitization. Confirm no prohibited packets, destructive writes, raw binaries, captures, or secrets enter the contribution.
4. Check device matching evidence beyond VID/PID. Inspect capabilities and platform/firmware limits, distinguishing simulation, inference, executed software tests, and physical validation. Reject unsupported claims.
5. Verify the commands actually executed and outcomes. Run relevant checks only when tooling and workspaces exist; inspect CI content rather than assuming a workflow filename means functioning CI. Record unavailable checks explicitly.
6. Check relative links, skill frontmatter, public contracts/ADRs, English/pt-BR consistency, licensing/provenance, and the status ledger. Apache-2.0 remains proposed until maintainer confirmation; do not invent contacts or CODEOWNERS.
7. Report findings by severity with file/location, rationale, evidence, and actionable resolution. Distinguish blockers, scope limitations, and optional improvements. Prepare a human review recommendation without auto-publishing.

## Artifacts

A review summary, prioritized findings, actual validation record, capability/evidence gaps, and acceptance conditions. A documentation review is not a security audit or physical compatibility certificate.

## Stop conditions

Withhold acceptance for fabricated results, missing required evidence, exposed private data, unsafe execution, unresolved license/provenance, or unauthorized scope. If Git/tools are unavailable, report the exact review limitation rather than inventing a diff or CI result. Opening a PR, pushing, or publishing requires explicit authorization.
