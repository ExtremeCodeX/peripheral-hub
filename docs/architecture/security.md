# Security architecture

**Status: required controls; not implemented or audited.** The public [security policy](../../SECURITY.md) governs reporting and responsible research.

## Assets and untrusted inputs

Protect physical devices, local user data/profiles, credentials, private research, typed input, and device identifiers. Treat device responses, imported profiles/reports, manifests, vendor artifacts, and frontend requests as untrusted. A community driver is reviewed source, not a license to bypass validation.

## Required controls by boundary

| Boundary | Required control |
| --- | --- |
| React → Tauri | Least-privilege capabilities and narrow typed API; no raw USB/HID access |
| Tauri → daemon | Versioned same-user local IPC, bounds/timeouts, explicit errors |
| Registry → driver | Evidence-backed matching, ambiguity handling, granular capability checks |
| Manifest → interpreter | Typed/schema/semantic validation and allowlisted bounded operations |
| Driver → transport | Valid report types/lengths, rate/timeout constraints, truthful failures |
| Research → public artifact | Authorization, minimal evidence, sanitization, provenance, human review |
| Persistence → export/import | Schema versions, migrations, limits, no proprietary raw evidence |

## Laboratory isolation

Unknown vendor programs are never executed automatically. Dynamic analysis and elevated capture require explicit authorization, an appropriate laboratory, and attention to VM passthrough limitations. Raw binaries/captures stay in ignored local storage; no automatic uploads to AI or other external services.

Disallowed operations include unknown packet replay, bootloader/firmware actions, undocumented permanent writes, and destructive commands. Global input hooks/injection are outside the initial macro scope. Hardware testing is opt-in and separate from CI.

## Verification still required

Future software must verify IPC permissions/peer identity, parser and manifest bounds, dependency risks, Tauri grants, sensitive logs, migrations, and report sanitization. Hardware validation is capability/firmware/platform-specific. No private contact, audited runtime, signed release, or CI security job is currently available; pending items are in the [status ledger](implementation-status.md).
