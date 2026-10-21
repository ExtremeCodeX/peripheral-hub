# ADR 0012: Validated DRA handoff for AI agents

- Status: implemented as software with synthetic evidence only. No hardware access, capture, vendor execution, packet replay, external AI call, commit or publication is part of this decision.
- Date: 2026-10-21.

## Context

Phase 6 restored the standalone DRA: local JSON sessions, bounded textual capture import, deterministic analysis, sanitized Markdown/JSON reports and a commented scaffold. The project skills already describe responsible research, but they previously stopped at pointing contributors to the DRA reference. An AI agent needs a concrete, validated bridge from report evidence to a contribution skeleton without inventing commands, promoting inferred offsets or generating a loadable driver.

The master document's Phase 7 is release preparation, while the user explicitly authorized an AI-agent integration phase. Packaging, release, CI and publication remain unauthorized.

## Decisions

1. Add a separate workspace crate `tools/agent-handoff` with the `dra-agent` binary. It depends on the DRA and the real `peripheral-hub-driver-declarative` validator; it has no network, hardware, capture, daemon or desktop dependency.
2. Consume only the DRA's typed `ReportDocument`. Reject unknown fields, malformed JSON, unsupported versions and reports that fail `ReportDocument::validate()`.
3. Add cross-document checks: timestamp ordering, capture event totals, known groups, variable offsets, hypothesis references and payload bounds. Hash the exact report bytes with SHA-256.
4. Compute a deterministic readiness state (`insufficient`, `needs-more-evidence`, `scaffold-ready`, `hardware-review`) and required experiments. `create` refuses `insufficient`; all other states produce a review handoff that still forbids implementation without further evidence and authorization.
5. Generate a traceable, non-loadable contribution skeleton: `handoff.json`, `AGENT_HANDOFF.md`, `protocol.md`, `validation.md`, test/fixture guidance, a fully commented `manifest.toml.example`, and an exact copy of the sanitized report plus regenerated Markdown. No driver code, test, payload, capture or loadable manifest is generated.
6. Escape report-derived text in Markdown and state that report content is untrusted data, not agent instructions.
7. Validate optional manifests with the real declarative parser. Record only a structural result; never infer a report match, device match or hardware capability.
8. Provide `verify` to recompute the copied report digest and assessment, and `validate-manifest` for a non-generating manifest check.
9. Add a documentary JSON Schema for the handoff in `schemas/research-handoff.schema.json`; Rust typed checks remain authoritative.
10. Rewrite the six requested skills around executable `dra-agent`/DRA commands, add skill-compatibility tests, and document optional REA only from official sources without registering MCP or installing it.
11. Keep the tool standalone and local. It does not enter the shipped daemon/desktop runtime and is not an autonomous research agent.

## Consequences

The contribution path becomes:

```text
DRA session → analyze → report.json → dra-agent inspect → create → human/agent review
                                              ↘ verify
                                               ↘ validate-manifest (optional)
```

The handoff is reproducible and can be checked independently of a device. It improves traceability, prevents accidental report edits, and blocks a common failure mode where an inferred offset is treated as a driver specification.

The boundary remains explicit: no report check proves a protocol or hardware result; a manifest check is structural; a synthetic test proves software behavior only. REA remains optional and absent from the implementation. The added crate increases the workspace member count and test count; the exact results are recorded in the [status ledger](../implementation-status.md).

## Related documents

- [DRA reference](../dra.md)
- [Agent integration](../agent-integration.md)
- [Agent contribution workflow](../../reverse-engineering/agent-contribution-workflow.md)
- [Research workflow](../../reverse-engineering/research-workflow.md)
- [Handoff schema](../../../schemas/research-handoff.schema.json)
- [ADR 0011](0011-dra-mvp.md)
