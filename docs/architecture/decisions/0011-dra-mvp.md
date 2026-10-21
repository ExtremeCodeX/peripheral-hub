# ADR 0011: Standalone DRA MVP with textual captures and evidence labels

- Status: implemented as software (`apps/device-research-assistant`, local JSON sessions, analysis, sanitized reports and a commented scaffold); synthetic/software evidence only, no pcap parser, no hardware operation and no physical compatibility claim. Recovered into the current branch from `25a07f2` on 2026-10-21 without a scope change.
- Date: 2026-10-20.

## Context

The master's Phase 5 asks for a standalone Rust Device Research Assistant that helps researchers import captures, normalize events, compare repetitions, generate evidence-labeled hypotheses, export sanitized reports and scaffold a driver contribution. Phase 5 had been explicitly out of scope until the user authorized the MVP without hardware. The master also requires that the DRA not promise impossible results, that raw evidence stay in ignored local storage, that no unknown packet be replayed, and that `confirmed-on-hardware` never be fabricated. The repository already had an empty `apps/device-research-assistant` scaffold and an empty `schemas/research-report.schema.json` placeholder.

## Decisions

1. Add `device-research-assistant` as the twelfth Cargo member and standalone binary/library. It depends only on the SDK (descriptors), the HID transport crate (non-destructive inventory), `serde`, `serde_json` and `thiserror`; it has no daemon/IPC/desktop dependency and no network client.
2. Store sessions as local, versioned JSON files under `<user data>/peripheral-hub/dra/sessions/<id>/`, never inside the repository. Session and capture schema version is `1`; writes use a same-directory temporary file plus rename. Capture import is size-limited (16 MiB input, 20,000 events, 4,096 payload bytes, 32 captures) and normalizes by timestamp with lowercase hex.
3. Accept only a stable project JSON capture format with typed `direction`/`transfer` enums, optional report id and optional descriptive action label. Unknown fields fail. **No pcap/pcapng/TShark parser is implemented**, and no dependency or command is claimed for one. Raw captures and executable paths are not part of a session or report.
4. Analyze deterministically by grouping on `(direction, transfer, interface, report_id, payload_len)`, comparing outbound offsets, and checking whether labeled actions form a one-to-one value mapping. Emit `inferred` only for a consistent mapping and `unverified` otherwise. The model recognizes `confirmed-on-hardware` for human evidence but automatic analysis never emits it.
5. Export `report.md` and `report.json` with session metadata, group/offset summaries, hypotheses and limitations. The JSON shape is documented by a schema file, while the typed Rust validator and serializer are authoritative. Reports exclude raw payloads, input paths, serials and private locators.
6. Emit a scaffold only when at least one hypothesis exists. `manifest.toml.example` is fully commented so it cannot be loaded; the scaffold declares no capability and contains no driver code or raw capture. A non-empty output directory requires `--force`.
7. Expose `devices list` as optional, non-destructive HID enumeration that opens no handle and writes no report. It was not executed during validation; a synthetic descriptor test covers sanitization. No active capture, vendor execution or hardware authorization is included.
8. Keep the DRA out of the shipped desktop/daemon runtime. REA remains optional and unconfigured; the empty `config/agents/rea-mcp.example.json` was not filled with invented MCP commands.

## Consequences

The Phase 5 MVP is now a real local pipeline: create a consented session, import a synthetic textual capture, analyze it, produce sanitized Markdown/JSON reports, and generate a deliberately inert scaffold. Eight integration tests cover the happy path, rejection paths, normalization, consent/id safety, sanitized inventory and schema-file parsing. The full workspace test count increases by eight.

The boundary is explicit: there is no capture backend, pcap parser, packet replay, vendor execution, driver generation or hardware validation. Analysis is heuristic and cannot prove protocol semantics. A future pcap/TShark importer, richer correlation, REA documentation and driver handoff require separate design, tests and authorization. See the [DRA reference](../dra.md), [research workflow](../../reverse-engineering/research-workflow.md), [capture schema](../../../schemas/research-capture.schema.json), [report schema](../../../schemas/research-report.schema.json) and [status ledger](../implementation-status.md).
