---
name: capture-protocol
description: Plans authorized USB or HID capture sessions and correlates user actions with traffic when collecting evidence for a peripheral protocol.
compatibility: opencode
---

# Capture and correlate protocol evidence

## Inputs

Research goal, owner consent, exact device/interfaces, OS/firmware/vendor software versions, authorized capture tools, expected safe actions, privacy constraints, and the local storage location.

## Tools

Read the binding [responsible research policy](../../../docs/reverse-engineering/responsible-research.md), [SECURITY.md](../../../SECURITY.md), the [research workflow](../../../docs/reverse-engineering/research-workflow.md) and the [agent handoff guide](../../../docs/reverse-engineering/agent-contribution-workflow.md). The DRA is implemented for local sessions and the documented textual JSON format only. It does not parse pcap/pcapng, run live capture or access hardware beyond non-destructive inventory. USBPcap plus Wireshark/TShark may be used only in an authorized laboratory after checking installed versions; those formats are not DRA-supported. Do not invent import commands.

## Procedure

1. Verify consent, scope, target interface, storage location and any elevation authorization. Account for unrelated devices and sensitive keyboard/input traffic. Stop if the purpose or authorization is unknown.
2. Create a real local DRA session, or record equivalent facts manually if the CLI is unavailable:
   ```sh
   cargo run -p device-research-assistant -- session create \
     --name "NAME" --device "MODEL" --firmware "FW" \
     --goal "One interoperability question" --consent
   ```
3. Plan a baseline plus one known safe volatile action at a time, with repeated observations. Do not assume a vendor UI action is safe if its persistent effects are unknown. Unknown writes, firmware/bootloader entry and packet replay are out of scope.
4. Once separately authorized, capture only the necessary target traffic for a short interval. Record action timestamps separately. Do not start elevated capture automatically.
5. Sanitize before import: remove unrelated traffic, serials, usernames, private paths, credentials and typed input. Normalize to the [capture schema](../../../schemas/research-capture.schema.json). The importer rejects unknown fields and malformed data:
   ```sh
   cargo run -p device-research-assistant -- capture import \
     --session ID --input sanitized-capture.json
   cargo run -p device-research-assistant -- analyze --session ID
   cargo run -p device-research-assistant -- report --session ID
   ```
6. Gate the report before sharing:
   ```sh
   cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- inspect --report report.json
   cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- create \
     --report report.json --output local-data/handoff/ID
   cargo run -p peripheral-hub-agent-handoff --bin dra-agent -- verify \
     --handoff local-data/handoff/ID
   ```
   `insufficient` means no hypothesis; `needs-more-evidence` means refine the experiment. Do not implement a driver from either state.
7. Review hypotheses and competing explanations: initialization, framing, ACK, timing, checksums and polling/heartbeat. Use `confirmed-on-hardware`, `inferred` or `unverified` according to actual evidence. Observing a UI action is not driver validation.
8. Hand the sanitized handoff to [develop-driver](../develop-driver/SKILL.md) or [validate-device](../validate-device/SKILL.md); record actions that were not authorized or not performed.

## Artifacts

Ignored local sessions and raw material, an action timeline, sanitized normalized evidence, comparison findings, confidence labels, required experiments, and the `dra-agent` handoff with matching digest. Synthetic captures must be labeled and cannot establish physical support.

## Failure handling

If sanitization removes too much to analyze, regenerate from the original locally rather than adding private data. If the report is contradictory, regenerate it with the current DRA. If `verify` fails, preserve the original and investigate; never edit the digest. If a parser/backend is unavailable, record that limitation and stop rather than claiming support.

## Stop conditions

Stop on missing consent/elevation approval, any prohibited activity, unknown purpose/authorization, unrelated sensitive capture, uncertain command safety, unexpected behavior, ambiguous interface, or unavailable tooling. Never replay unknown packets, execute vendor binaries automatically, or write firmware/persistent memory. Completion requires traceable correlation and reviewed exports, not a complete protocol claim.
