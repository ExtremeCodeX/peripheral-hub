# Responsible peripheral research workflow

**Current status:** DRA, report schemas, capture parsers, and scaffold utilities are not implemented. This is a human/agent research procedure and future tooling requirement, not a CLI manual.

## Mandatory interoperability boundary

Read the [responsible research policy](responsible-research.md) before any step. This workflow serves only authorized peripheral protocol study, interoperability and independent driver development. No bypass, secret extraction, exploit development, unauthorized access or proprietary source reproduction is allowed. If the purpose or authorization is unknown, suspend the affected operation and request human review. Permission for an active experiment never waives these restrictions.

## 1. Authorization and local session

Read [SECURITY.md](../../SECURITY.md). Confirm owned/adequately authorized hardware, legitimate executable provenance, copyright/license/law compliance, and an explicit interoperability question. Prioritize static inspection and passive USB/HID observation. Record date/session ID, target interfaces, OS, firmware/revision, vendor version/SHA-256, goal, tools, consent scope and limitations in ignored local storage. `local-data/` is the recommended location. Do not automatically upload original binaries or raw captures to a model or external service.

## 2. Static analysis

Inspect metadata and relevant code/data with available authorized local tools, without executing the sample. Optional [REA](https://github.com/morluto/rea) can help, but its installation has not been verified and it is not required by this foundation. Verify actual installation/startup documentation before enabling MCP. The empty `config/agents/rea-mcp.example.json` is not usable configuration.

Look only for protocol-relevant API calls, interface matching, report construction, initialization, lengths, checksums, sequencing and error handling. Separate observations from inference. An executable may be an installer/wrapper and may not reveal the whole protocol. Do not seek security bypasses, credentials, private keys or confidential manufacturer information. Do not reproduce proprietary source, including any decompiled snippets, in code, tests, docs or reports; independently implement only permissible protocol facts.

## 3. Controlled dynamic capture

Vendor execution, physical experiments and elevated capture are separately authorized laboratory actions, after static/passive methods have been considered. Before an active physical action, record explicit human approval for the exact device/interface, purpose, action, limits and stop conditions. Ownership, discovery and tooling do not authorize it. Use an isolated laboratory machine/VM where appropriate; physical USB passthrough can be constrained. Never run a sample automatically or perform any prohibited action even after approval.

On Windows, USBPcap with Wireshark/TShark is a candidate capture path. Inspect installed versions and official tool instructions before capture; elevation may be needed for capture but must not become an application requirement. Filter to the target device/interface and avoid unrelated input. Record baseline and one known safe volatile vendor-UI change at a time with timestamps; do not replay unknown packets.

Keep captures short, preserve raw input locally, and stop on unexpected behavior or privacy leakage. Vendor controls with unknown permanent effects are outside this workflow.

## 4. Import and correlation

Future DRA should support a stable project JSON capture format for CI and only claim pcap/pcapng support when a parser/backend is implemented and tested. TShark structured exports require a verified mapping and documented dependency version. No import commands or field schema are available yet.

The conceptual normalized event needs timestamp, direction, interface, transfer/report type, report ID, payload, origin, and correlated action. Compare repetitions and action differences; retain alternative explanations, timing, framing, and unknowns. Use `confirmed-on-hardware`, `inferred`, and `unverified` for hypotheses with scoped evidence.

## 5. Sanitization and handoff

Keep originals/raw captures local. Remove unrelated traffic, typed input, serials, usernames and private paths from working exports. If credentials/private keys/confidential information are encountered, suspend that analysis and request review instead of extracting them. Export only minimum authorized, reviewed, sanitized protocol findings; exclude all proprietary source. Synthetic data must be labeled and cannot prove physical compatibility.

Write findings using the [protocol documentation format](../protocols/specification-format.md). Hand off identity, capabilities, bounds, provenance, and uncertainties to the [driver workflow](../../.agents/skills/develop-driver/SKILL.md). Actual driver development needs implemented SDK/validators and separate scope authorization.

## Completion and limits

A scoped session can end with useful unanswered questions. Unknown purpose/provenance/authorization or a prohibited request means stop that operation and request human review; do not solve the blocker through bypass, exploitation or copying vendor code. Missing hardware/OS/tools/DRA/REA must be documented. Capturing a vendor RGB action does not prove a project driver works, and static analysis is never physical validation.
