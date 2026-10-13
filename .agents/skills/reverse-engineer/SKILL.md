---
name: reverse-engineer
description: Guides local analysis of authorized vendor executables and dependencies when researching a peripheral protocol, using optional REA and explicit evidence labels.
compatibility: opencode
---

# Reverse engineer authorized artifacts

## Inputs

Vendor artifact source, authorization, local path, SHA-256, version, target device/interface, research questions, and any known capture evidence. Never request that a proprietary sample be committed or uploaded.

## Tools

Read [SECURITY.md](../../../SECURITY.md) and the [research procedure](../../../docs/reverse-engineering/research-workflow.md). Use locally available metadata, hash, string/import inspection, or decompilation tools. [REA](https://github.com/morluto/rea) is optional. Verify its current installation and startup instructions before MCP use; the empty repository example is not configured.

## Procedure

1. Check user scope and authorization. Record provenance and hash without executing the sample. Keep original binaries and derived sensitive output in ignored local storage.
2. Identify packaging/runtime and relevant dependencies locally. Distinguish wrappers, installers, and embedded application components; do not assume the executable contains the complete protocol.
3. Inspect static references to HID/USB APIs, report IDs, lengths, command construction, model matching, initialization, checksums, and response handling. Track each observation to a reproducible artifact location/version.
4. Separate observed code/data from interpretation. Mark protocol hypotheses `inferred` or `unverified`; static analysis cannot yield `confirmed-on-hardware` on its own.
5. Correlate existing sanitized capture evidence where available. Suggest descriptive experiments through [capture-protocol](../capture-protocol/SKILL.md), without replaying unknown packets.
6. Dynamic analysis is a separate explicitly authorized laboratory step. Confirm isolation, permissions, physical device constraints, and collection scope before any vendor program runs.
7. Export only necessary sanitized findings, uncertainties, and safe capability implications. Record unavailable tools and limitations rather than inventing REA commands or results.

## Artifacts

An analysis summary with non-sensitive provenance, artifact hash/version, tool versions, facts versus hypotheses, evidence references, protocol questions, and recommended capture actions. No proprietary executable, dump, protected source excerpt, private paths, or serials in public artifacts.

## Stop conditions

Stop on missing legal authorization, required external upload, unapproved execution/elevation, unsafe hardware actions, or insufficient tool support. No invented packet formats or physical compatibility claims. Completion means the scoped questions have documented evidence or explicit unresolved status, not a guarantee that a driver can be built.
