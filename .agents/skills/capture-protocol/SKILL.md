---
name: capture-protocol
description: Plans authorized USB or HID capture sessions and correlates user actions with traffic when collecting evidence for a peripheral protocol.
compatibility: opencode
---

# Capture and correlate protocol evidence

## Inputs

Research goal, owner consent, exact device/interfaces, OS/firmware/vendor software versions, available capture tools, expected safe actions, and privacy constraints.

## Tools

Read the binding [responsible research policy](../../../docs/reverse-engineering/responsible-research.md), [SECURITY.md](../../../SECURITY.md) and the [research workflow](../../../docs/reverse-engineering/research-workflow.md). Captures serve only authorized interoperability study; keep originals local, never upload automatically, and stop for human review when purpose/authorization is unknown. The DRA MVP is implemented but accepts only its documented textual JSON capture format; it does not parse pcap/pcapng, run live capture or access hardware. USBPcap plus Wireshark/TShark may be used in an authorized Windows laboratory after availability/version checks. Do not present those formats as DRA-supported or invent unimplemented import commands.

## Procedure

1. Verify consent, scope, target interface, storage location, and any capture elevation authorization. Account for unrelated devices and sensitive keyboard/input traffic.
2. Create a local DRA session with `cargo run -p device-research-assistant -- session create --name … --device … --goal … --consent`, or record the same facts manually if the CLI is unavailable: identifier/date, goal, device/firmware, OS, vendor source/version/hash, consent scope, tools, and raw evidence paths.
3. Plan descriptive vendor-UI actions: baseline, one known volatile color/brightness change at a time, and repeated observations. Do not infer that a vendor UI action is safe if its persistent effects are unknown.
4. Once authorized, capture only the necessary target traffic in a short controlled interval. Record action timestamps separately. Use verified installed-tool instructions; do not start elevated capture automatically.
5. Preserve raw evidence locally. For analysis, normalize timestamp, direction, interface, transfer/report type, report ID, payload length, source, and correlated action; keep unknown fields explicitly unknown.
6. Compare repeated sequences and changing offsets. Check initialization, framing, ACKs, timing, checksums, and competing explanations. Mark hypotheses `confirmed-on-hardware`, `inferred`, or `unverified` according to actual evidence; observing a UI action is not driver validation.
7. Sanitize minimal exports: remove unrelated traffic, serials, private paths, keys, typed data, and identifying payloads. Review before sharing. Use DRA's documented JSON capture format; do not assume pcap compatibility.
8. Hand a scoped report and uncertainties to [develop-driver](../develop-driver/SKILL.md); record dependencies and actions that could not be performed.

## Artifacts

Ignored local session/captures, an action timeline, sanitized normalized evidence, comparison findings, confidence labels, and a report with exact environment/tool versions. Synthetic captures must be explicitly labeled and cannot establish physical support.

## Stop conditions

Stop on missing consent/elevation approval, any prohibited activity, unknown purpose/authorization requiring human review, unrelated sensitive capture, uncertain command safety, unexpected behavior, ambiguous interface, or unavailable parser/backend. Never replay unknown packets, execute vendor binaries automatically, or write firmware/persistent memory. Completion requires traceable correlation and reviewed exports, not a complete protocol claim.
