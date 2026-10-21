# AI agent and DRA integration

**Status: implemented as a local software tool, validated with synthetic DRA evidence only.** The `peripheral-hub-agent-handoff` crate and `dra-agent` binary turn a validated, sanitized DRA report into a traceable review handoff. They do not generate driver code, a loadable manifest, a test, a capture or a capability claim. No hardware, vendor executable, packet replay or external AI service was used.

The binding rules remain [AGENTS.md](../../AGENTS.md), the [responsible research policy](../reverse-engineering/responsible-research.md) and [SECURITY.md](../../SECURITY.md). This integration is outside the shipped desktop/daemon runtime; REA remains optional and is not a dependency. See [ADR 0012](decisions/0012-agent-dra-handoff.md).

## Why a separate gate

The DRA already emits sanitized JSON/Markdown and a commented scaffold. A model can accidentally treat report text as instructions, overstate an inferred offset, or jump directly to a manifest. The handoff tool enforces a narrow, testable step:

```text
authorized research session
        ↓
DRA JSON capture import → analysis → report.json (typed, sanitized)
        ↓
dra-agent inspect  (typed + cross-document integrity, readiness)
        ↓
dra-agent create   (traceable copy + contribution skeleton)
        ↓
human/agent review → separately authorized driver work
        ↓
dra-agent validate-manifest (real declarative parser, structural only)
```

The tool is deliberately not an autonomous reverse-engineering agent. It has no network client, no capture backend, no hardware access, no vendor execution and no model API. The human or agent still reads the evidence and decides what, if anything, can responsibly be implemented.

## Commands

`dra-agent` is a separate binary in the root workspace:

```sh
cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- --help
cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- inspect --report report.json
cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- create \
  --report report.json --output local-data/handoff/<session-id>
cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- create \
  --report report.json --output local-data/handoff/<session-id> \
  --manifest candidate.toml
cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- verify \
  --handoff local-data/handoff/<session-id>
cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- validate-manifest \
  --manifest candidate.toml
```

All commands accept `--json` for automation. `create` refuses an existing non-empty output directory unless `--force` is supplied; `--force` overwrites known generated files but does not delete unrelated files. `local-data/` is ignored by Git.

`inspect` validates the report but writes nothing. `create` refuses a report with readiness `insufficient`. `verify` re-reads the copied report, recomputes its digest and assessment, and rejects modifications. `validate-manifest` invokes `peripheral-hub-driver-declarative` directly; success is structural only.

## Report validation

The tool reads at most 32 MiB and deserializes the DRA's typed `ReportDocument` with unknown fields denied. It then calls `ReportDocument::validate()` and adds cross-document checks that the DRA model does not perform by itself:

- analysis and report timestamps do not predate the session, and the report does not predate its analysis;
- capture event counts sum to the analyzed event total;
- every variable offset belongs to a known outbound group, lies inside the payload, has unique observed values and keeps labeled action values inside that set;
- action-offset hypotheses reference a known outbound group, a valid offset and a matching variable offset with labeled observations;
- variable-offset hypotheses reference a known outbound group with a matching offset;
- repeated-payload hypotheses agree with their group summary (one distinct payload, at least two observations);
- duplicate or contradictory hypothesis keys are rejected;
- capture counts and bounds remain within DRA limits.

The exact report bytes are hashed with SHA-256. A generated handoff copies those bytes to `evidence/report.json`; `verify` recomputes the digest and the complete deterministic assessment. A tampered report, unknown schema version, malformed JSON or contradictory hypothesis is rejected before a handoff is claimed valid.

This validates software consistency, not physical truth. A report saying `confirmed-on-hardware` is a human provenance claim; the tool cannot verify that an experiment occurred. It never promotes labels and never creates that label.

## Readiness states

| State | Meaning | `create` behavior |
| --- | --- | --- |
| `insufficient` | No hypothesis exists | Refused; collect a baseline and one labeled change |
| `needs-more-evidence` | Only unverified observations exist | Allowed, with a gate that forbids implementation until refined |
| `scaffold-ready` | At least one inferred correlation exists | Allowed as a clearly labeled scaffold only |
| `hardware-review` | A confirmed-on-hardware hypothesis is present | Allowed only for human review of provenance |

The generated `required_experiments` are deterministic and include the evidence-specific next step, authorization and no-replay reminders. Presence of a manifest does not raise readiness or prove a match.

## Generated handoff

```text
<output>/
  handoff.json              # schema 1, typed provenance + assessment
  AGENT_HANDOFF.md          # gate, evidence summary, required work
  protocol.md               # inferred/unverified observations only
  validation.md             # empty matrix; all real checks remain "not run"
  tests/README.md           # no tests generated
  fixtures/README.md        # no payload or capture fixtures generated
  manifest.toml.example     # every non-empty line commented out
  evidence/
    report.json             # exact validated report bytes
    report.md               # regenerated sanitized Markdown
```

`handoff.json` records the report schema/session schema versions, timestamps, report SHA-256, device and capture metadata, hypotheses, counts, required experiments, warnings and explicit safety booleans. It is documented by [research-handoff.schema.json](../../schemas/research-handoff.schema.json); Rust typed checks in `dra-agent` remain authoritative.

The Markdown files escape report-derived text before display. Report text is **data**, not instructions. The generated manifest example is not loadable and declares no model, match, capability or operation.

## Evidence boundaries

- No full payloads, raw captures, input paths, serial numbers, credentials or vendor binaries are copied. The DRA report is already sanitized; the handoff copies only that report plus regenerated Markdown.
- No driver code, executable test or fixture is generated.
- No capability is declared or implied. `scaffold-ready` means "correlated observations may be documented", not "the device supports this".
- No vendor executable is run, no packet is sent or replayed and no device is opened.
- A passing manifest validation does not prove report agreement, device identity, firmware compatibility or physical behavior.
- Physical experiments require explicit human authorization for the exact device, action and limits.

## Validation evidence

The integration tests invoke the **real DRA CLI** to create a synthetic consented session, import a synthetic capture, and write `report.json`; then they exercise `dra-agent`:

- valid report → `scaffold-ready` handoff with copied digest, commented template, no raw hex, and successful `verify`;
- unknown report field, event-total disagreement, unknown group, duplicate hypothesis and tampered copied report → rejected;
- insufficient report → no output directory created;
- unverified-only report → `needs-more-evidence` gate;
- non-empty output without `--force` → refused; forced create preserves unrelated files;
- real declarative example passes structural validation, and a future manifest version fails;
- every requested skill has valid frontmatter, required headings, the shared workflow link and executable DRA/handoff commands.

The exact suite count and full workspace result are recorded in the [status ledger](implementation-status.md). These tests prove local parsing, consistency, generated structures and CLI behavior only. They do not validate a device, protocol, manifest match or hardware capability.

## Optional REA

REA is **not installed or verified in this environment**. The official project documentation at `https://github.com/morluto/rea` describes an agent setup command (`npx rea-agents setup`); no installation, MCP registration or artifact execution was performed. `config/agents/rea-mcp.example.json` remains empty. If a contributor verifies an installation in their own authorized environment, REA can assist static analysis, but its findings still require provenance and evidence labels. It is not required for this handoff and must not be presented as bundled.

## Limitations

1. The tool consumes only the DRA's typed report format; it cannot read pcap/pcapng, live captures or arbitrary vendor formats.
2. Integrity checks are structural and cross-document; they cannot establish that an experiment happened or that a protocol meaning is correct.
3. The readiness gate is conservative but heuristic. Human review remains mandatory. `verify` rechecks the report and assessment, but the optional manifest summary is not re-verifiable because the candidate manifest is intentionally not copied; run `validate-manifest` on the candidate again.
4. The handoff skeleton is not a driver package and is not registered with the daemon.
5. No GitHub, PR, CI, packaging or external AI integration is implemented.
6. No hardware or proprietary artifact was used in validation.
