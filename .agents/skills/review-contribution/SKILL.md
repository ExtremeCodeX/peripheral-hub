---
name: review-contribution
description: Reviews documentation, feature, or driver changes for architectural safety, evidence, compatibility, licensing, and truthful validation before human acceptance.
compatibility: opencode
---

# Review a contribution

## Inputs

Requested review scope, actual change set/base when Git is available, rationale, affected contracts, check results, `dra-agent` handoff and its report digest, sanitized evidence, and known limitations.

## Tools

Read the binding [responsible research policy](../../../docs/reverse-engineering/responsible-research.md), [CONTRIBUTING.md](../../../CONTRIBUTING.md), [SECURITY.md](../../../SECURITY.md), [AGENTS.md](../../../AGENTS.md), the [status ledger](../../../docs/architecture/implementation-status.md) and the [agent integration](../../../docs/architecture/agent-integration.md). Inspect files/diffs, real tests/validators and provenance. Verify the current Git state rather than relying on an earlier audit.

## Procedure

1. Identify actual changes and pre-existing work. Compare requested scope with delivery; flag placeholders represented as implementations, unrequested release/CI work, or code introduced during documentation-only work.
2. Review architecture boundaries: restricted frontend, same-user versioned IPC, least privilege, compiled reviewed drivers, bounded declarative execution, distinct model/instance identity, and a standalone DRA/handoff outside the shipped runtime.
3. For an AI/DRA handoff, verify the gate rather than trusting prose:
   ```sh
   cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- verify --handoff local-data/handoff/ID
   ```
   Check that `handoff.json` and `evidence/report.json` agree, the report is sanitized, no raw capture/payload/vendor binary is present, and `manifest.toml.example` has no uncommented lines.
4. Inspect parsers, command limits, unsupported errors, logs, transport failures, migrations, dependency changes and fixture sanitization. Confirm no prohibited packets, destructive writes, raw binaries, credentials or personal data enter the contribution.
5. Check identity beyond VID/PID and capability-level evidence. Reject claims that an inferred report, synthetic test or manifest parser pass proves hardware support. Confirm `insufficient`/`needs-more-evidence` gated implementation.
6. Verify the commands actually executed and outcomes, running relevant checks only when tooling exists. Do not treat an absent CI file as active CI or a synthetic test as physical validation.
7. Check relative links, skill frontmatter, public contracts/ADRs, English/pt-BR consistency, licensing/provenance and the status ledger. Apache-2.0 remains proposed until maintainer confirmation; do not invent contacts or ownership.
8. Report findings by severity with file/location, rationale, evidence and resolution. Distinguish blockers, scope limitations and optional improvements. Prepare a human review recommendation without auto-publishing.

## Artifacts

A review summary, prioritized findings, actual validation record, verified handoff digest, capability/evidence gaps and acceptance conditions. A documentation/software review is not a security audit or physical compatibility certificate.

## Failure handling

If the handoff digest, assessment or report copy fails verification, treat it as a blocker. If Git/tools are unavailable, report the exact limitation rather than inventing a diff or CI result. If a test output is claimed without a command/result, request evidence and do not accept it.

## Stop conditions

Withhold acceptance for fabricated results, missing required evidence, exposed private data, unsafe execution, unresolved license/provenance, unauthorized scope, or any prohibited reverse-engineering activity. Opening a PR, pushing or publishing requires explicit authorization.
