# Device Research Assistant (DRA)

**Status: standalone MVP and a validated agent handoff implemented as software, exercised with synthetic captures only.** The DRA imports authorized textual captures, groups repeated transfers, highlights varying payload offsets, correlates explicitly labeled actions, and exports sanitized Markdown/JSON reports plus a deliberately non-loadable driver scaffold. It does **not** execute vendor binaries, send or replay packets, parse pcap/pcapng, or control hardware. There is no physical validation claim. The original implementation was preserved in `backup/fases-5-6` at `25a07f2` and restored in the current Phase 6 branch without copying Phase 5 domain code. See [ADR 0011](decisions/0011-dra-mvp.md), [ADR 0012](decisions/0012-agent-dra-handoff.md), the [agent integration](agent-integration.md) and the [status ledger](implementation-status.md).

The binding rules are in the [responsible research policy](../reverse-engineering/responsible-research.md) and [AGENTS.md](../../AGENTS.md). The DRA is a development tool, is absent from the shipped desktop/daemon runtime, and optional REA is not a dependency.

## Scope and trust boundary

```text
authorized local artifacts
        ↓
DRA JSON capture import (text, bounded)
        ↓
normalization → grouping → offset comparison → labeled hypotheses
        ↓
sanitized Markdown/JSON report + commented scaffold
        ↓
dra-agent inspect/create/verify → traceable handoff
        ↓
human review → separately authorized driver work
```

The DRA has its own data directory and never writes inside the repository by default. Raw executables, pcap files and private captures stay in ignored local storage; only reviewed, sanitized textual JSON is imported. The DRA has no network client, no vendor execution and no device write path.

`dra devices list` is the only command that may enumerate hardware. It calls the project's non-destructive HID discovery, opens no handle and writes no report. It was not executed during this implementation's validation because no hardware was authorized or needed; a synthetic descriptor test covers its sanitized output instead.

## Commands

```text
dra devices list [--json]
dra session create --name NAME --device MODEL --goal TEXT --consent
                   [--id ID] [--firmware FW] [--software NAME] [--sha256 HEX] [--json]
dra session list [--json]
dra session show ID [--json]
dra capture import --session ID --input FILE [--json]
dra analyze --session ID [--json]
dra report --session ID [--output DIR] [--json]
dra scaffold --session ID [--output DIR] [--force] [--json]
dra --data-dir PATH ...
```

`dra` below is shorthand for `cargo run -p device-research-assistant --`; Cargo builds the binary under the package name `device-research-assistant`, and the CLI's own help text uses `dra` for brevity. `--help` prints the same command list, the boundaries and the resolved data root.

- Session creation requires `--consent`; it records the user's authorization assertion, not proof of ownership.
- `--id` is optional and path-safe. Generated ids combine a name slug with the local creation time.
- `--software` and `--sha256` are optional metadata. `--sha256` requires `--software` and exactly 64 hex characters.
- `analyze` and `report` require at least one imported capture. `report` recomputes the analysis so a stale analysis cannot be exported silently.
- `scaffold` requires at least one hypothesis and refuses a non-empty output directory unless `--force` is passed. It never deletes unrelated files.

## Local layout and defaults

The default root is `<per-user data>/peripheral-hub/dra`:

- Linux: `$XDG_DATA_HOME/peripheral-hub/dra`, falling back to `~/.local/share/peripheral-hub/dra`.
- Windows: `%LOCALAPPDATA%/peripheral-hub/dra`.
- Fallback when no home/app-data variable exists: the system temporary directory.
- `PERIPHERAL_HUB_DRA_DATA` overrides the exact root; `PERIPHERAL_HUB_DATA` supplies the parent, matching the daemon convention; `--data-dir` overrides both for one command.

```text
<root>/sessions/<id>/
  session.json          # session metadata, schema 1
  capture-0001.json     # normalized capture, schema 1
  analysis.json         # derived analysis, schema 1
  report.md             # sanitized Markdown
  report.json           # sanitized JSON, schema 1
  scaffold/
    README.md
    manifest.toml.example
    tests/README.md
    hypotheses.json
```

Writes use a same-directory temporary file followed by a rename. Capture import writes the normalized file before updating `session.json`; on a session-write failure the new capture file is removed. Session directories are created exclusively, and ids cannot contain path separators or `..`.

## Capture import format

`capture import` accepts only the stable project JSON format. The implementation does **not** claim pcap, pcapng or TShark support. A minimal document:

```json
{
  "capture_schema_version": 1,
  "source": "synthetic",
  "description": "baseline then one safe RGB UI change at a time",
  "events": [
    {
      "timestamp_us": 10,
      "direction": "out",
      "transfer": "interrupt",
      "interface": 0,
      "report_id": 1,
      "payload_hex": "1100aa00",
      "action": "baseline"
    },
    {
      "timestamp_us": 20,
      "direction": "out",
      "transfer": "interrupt",
      "interface": 0,
      "report_id": 1,
      "payload_hex": "1101aa00",
      "action": "set-red"
    }
  ]
}
```

Fields are validated semantically before anything is persisted:

| Rule | Limit / behavior |
| --- | --- |
| Schema version | Must be `1`; future versions are rejected |
| Source | Lowercase token, 1–64 characters (`synthetic`, `tshark-json`, `manual`, …); never a path |
| Events | 1–20,000 per capture; at most 32 captures per session; 100,000 events analyzed in total |
| Payload | Hex with optional ASCII whitespace; even digit count; at most 4,096 decoded bytes |
| Direction | `in` or `out` |
| Transfer | `control`, `interrupt` or `bulk` |
| Interface / report id | `u8`; a missing report id is distinct from report id `0` |
| Action | Optional descriptive non-control label, 1–128 characters; not captured input |
| Input file | At most 16 MiB; UTF-8 JSON; unknown fields are rejected |
| Normalization | Events are sorted by `(timestamp_us, original index)`; lowercase hex is stored; reordering is recorded in session metadata |

The [capture schema](../../schemas/research-capture.schema.json) documents the shape. The Rust validator is authoritative; the schema file is a reference, not a substitute for import validation.

## Analysis and evidence labels

The analysis groups events by direction, transfer, interface, report id and payload length. For outbound groups it compares each payload offset across events and records the distinct byte values. When imported action labels are present, it checks whether each labeled action maps to exactly one value and each value maps back to exactly one action.

- **`inferred`**: the labeled actions show a one-to-one offset mapping. This is still a correlation, not physical proof.
- **`unverified`**: offsets vary without a one-to-one mapping, or a repeated identical outbound payload has no confirmed purpose.
- **`confirmed-on-hardware`**: recognized by the data model for human-authored evidence, but **automatic analysis never emits it**.

The analysis also records limitations: correlation-only evidence, no automatic hardware confirmation, no replay/transmission, no framing/checksum/sequencing/timing model, and any truncation at the retained-item limits. No full payload is copied into the report or scaffold.

## Reports and scaffolding

`report` writes `report.md` and `report.json` (or both into `--output DIR`). The Markdown report contains session metadata, capture counts, transfer groups, varying offsets, hypotheses with labels, and limitations. The JSON report is the typed `ReportDocument` and is documented by the [report schema](../../schemas/research-report.schema.json). Neither report includes raw capture paths, serial numbers, private locators or full payloads.

`scaffold` writes:

- `README.md` — evidence summary and the next review steps.
- `manifest.toml.example` — **every non-empty line is a comment**, so it cannot be loaded as a driver. It declares no capability or operation.
- `tests/README.md` — guidance for synthetic fixtures and negative-path tests.
- `hypotheses.json` — inferred/unverified hypotheses without payload contents.

This is an evidence handoff, not driver generation. A driver still requires separate authorization, reviewed protocol facts, bounded implementation and hardware validation.

## Agent handoff

The separate `peripheral-hub-agent-handoff` crate provides `dra-agent`, which consumes a validated `report.json` and writes a traceable review directory. It does not generate driver code or a loadable manifest and does not access hardware or the network.

```text
dra-agent inspect --report report.json [--json]
dra-agent create --report report.json --output DIR [--manifest FILE] [--force] [--json]
dra-agent verify --handoff DIR [--json]
dra-agent validate-manifest --manifest FILE [--json]
```

`inspect` applies the DRA typed validation plus cross-document consistency checks, computes a readiness state and lists required experiments. `create` refuses `insufficient` evidence and writes `handoff.json`, `AGENT_HANDOFF.md`, `protocol.md`, `validation.md`, test/fixture guidance, a fully commented manifest example, and an exact SHA-256-tracked copy of the sanitized report. `verify` recomputes the digest and assessment. Optional manifest validation uses the real declarative parser and proves structure only.

See the [agent integration reference](agent-integration.md) and [agent-assisted workflow](../reverse-engineering/agent-contribution-workflow.md) for commands, readiness meanings and safety boundaries.

## Validation evidence

Run from the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --locked
cargo test --workspace --locked
```

The DRA integration suite (`cargo test -p device-research-assistant`) covers: a synthetic session with an inferred offset and an unverified repeated payload; rejected future versions, malformed hex, unknown fields, unsafe source tokens, oversized payloads and empty events without session mutation; consent and path-safe id enforcement; out-of-order normalization; serial/path removal from inventory rows; schema-file parsing; and report/scaffold safety (no uncommented manifest lines, no payload or input-path leakage). The Phase 6 recovery rerun passed all **8 DRA tests** and the full workspace **111 tests**; a local CLI smoke run produced the synthetic report and commented scaffold without enumerating or opening a device. Phase 7 adds the handoff integration suite described in the [agent integration](agent-integration.md); exact workspace totals and the DRA rerun are recorded in the [status ledger](implementation-status.md).

## Limitations and pending work

1. **No pcap/pcapng or TShark parser.** Only the documented JSON format is implemented.
2. **No REA integration or MCP configuration.** REA remains optional, unverified tooling; no MCP configuration was invented.
3. **No hardware operation.** `devices list` enumerates non-destructively on demand; it was not run in this validation. No device was opened, written or replayed.
4. **No confirmed-on-hardware generation.** A human must attach hardware evidence separately.
5. **Heuristic analysis only.** It groups identical-shaped transfers and variable offsets; it does not model checksums, framing, timing, state machines or multi-packet transactions.
6. **No driver generation.** The scaffold and handoff are commented skeletons designed to prevent accidental capability claims. Manifest validation is structural only.
7. **No packaging or CI.** The DRA and handoff run from Cargo source; release packaging and automated CI remain out of scope.

See the [research workflow](../reverse-engineering/research-workflow.md) for the human procedure around these commands.
