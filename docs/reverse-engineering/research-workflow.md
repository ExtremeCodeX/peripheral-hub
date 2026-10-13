# Responsible peripheral research workflow

**Current status:** DRA, report schemas, capture parsers, and scaffold utilities are not implemented. This is a human/agent research procedure and future tooling requirement, not a CLI manual.

## 1. Authorization and local session

Read [SECURITY.md](../../SECURITY.md). Confirm lawful source/permission for software and hardware, research scope, and consent. Record date/session ID, target interfaces, OS, firmware/revision, vendor software version/SHA-256, goal, tools, consent scope, and limitations in ignored local storage. `local-data/` is the recommended repository-root location; it is not created by this stage.

## 2. Static analysis

Inspect metadata and relevant code/data with available authorized local tools, without executing the sample. Optional [REA](https://github.com/morluto/rea) can help, but its installation has not been verified and it is not required by this foundation. Verify actual installation/startup documentation before enabling MCP. The empty `config/agents/rea-mcp.example.json` is not usable configuration.

Look for API calls, interface matching, report construction, initialization, lengths, checksums, sequencing, and error handling. Separate reproducible observations from inference. An executable may be an installer/wrapper and may not reveal the whole protocol. Do not include protected binary content or extensive proprietary decompilation in public findings.

## 3. Controlled dynamic capture

Vendor execution, physical experiments, and elevated capture are separately authorized laboratory actions. Use an isolated machine/VM where appropriate, recognizing that physical USB passthrough can be constrained. Never run a sample automatically.

On Windows, USBPcap with Wireshark/TShark is a candidate capture path. Inspect installed versions and official tool instructions before capture; elevation may be needed for capture but must not become an application requirement. Filter to the target device/interface and avoid unrelated input. Record baseline and one known safe volatile vendor-UI change at a time with timestamps; do not replay unknown packets.

Keep captures short, preserve raw input locally, and stop on unexpected behavior or privacy leakage. Vendor controls with unknown permanent effects are outside this workflow.

## 4. Import and correlation

Future DRA should support a stable project JSON capture format for CI and only claim pcap/pcapng support when a parser/backend is implemented and tested. TShark structured exports require a verified mapping and documented dependency version. No import commands or field schema are available yet.

The conceptual normalized event needs timestamp, direction, interface, transfer/report type, report ID, payload, origin, and correlated action. Compare repetitions and action differences; retain alternative explanations, timing, framing, and unknowns. Use `confirmed-on-hardware`, `inferred`, and `unverified` for hypotheses with scoped evidence.

## 5. Sanitization and handoff

Remove unrelated traffic, typed input, serials, credentials, usernames, and private paths. Export only the minimum authorized sanitized evidence; review it before a PR or external sharing. Synthetic data must be labeled and cannot prove physical compatibility.

Write findings using the [protocol documentation format](../protocols/specification-format.md). Hand off identity, capabilities, bounds, provenance, and uncertainties to the [driver workflow](../../.agents/skills/develop-driver/SKILL.md). Actual driver development needs implemented SDK/validators and separate scope authorization.

## Completion and limits

A scoped research session can end with useful unanswered questions. Missing hardware, OS, authorization, capture tools, parsers, or REA must be documented. Capturing a vendor RGB action does not prove a project driver works, and static analysis is never physical validation.
