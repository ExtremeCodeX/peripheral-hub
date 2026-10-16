# ADR 0006: Bounded declarative drivers with an explicit interpreter

- Status: implemented as software (crate `peripheral-hub-driver-declarative`, a fictitious native example and daemon `--manifests` loading); no physical driver, hardware compatibility or DRA is claimed.
- Date: 2026-10-16.

## Context

The SDK, Registry, Core, transport, daemon/IPC and profiles existed, but the master document's §4.4 "drivers nativos e declarativos" had no declarative path: community manifests could not yet be parsed, validated or executed, `schemas/device-manifest.schema.json` was an empty placeholder, and no example driver existed. The user authorized implementing this driver system ("Fase 4" in their numbering; the master calls the user interface Fase 4 and the declarative manifest part of Fase 2). The SDK 1.0/Core/Registry contracts must not change.

## Decisions

1. Add the crate `peripheral-hub-driver-declarative`. A manifest is untrusted input parsed into typed structures; it may only select operations the interpreter already models. It cannot name a shell command, arbitrary bytes, a transport call, firmware/bootloader or a persistent-memory write. The only executable operation today is volatile `SetLighting` for one declared zone.
2. Keep the manifest a pure declaration. Matching reuses the SDK `DeviceMatchRule` (interface or HID-usage evidence beyond VID/PID is still required), capabilities reuse the SDK `Capabilities`, and encoding is deterministic: each field sets exactly one byte, offsets are unique and within the payload, lengths are 1..64 bytes, timeouts 0..30 s, rates 1..1000/s, `input` reports are rejected, and every declared effect must map to a unique byte.
3. Do not change the SDK contract. The interpreter's provenance is supplied by the factory (`DeclarativeFactory::new(manifest, mode)`), not read from the transport. The daemon constructs it with `Simulated` in `--simulate` and `Physical` otherwise, and the existing manager already rejects any driver outcome whose mode differs from the source mode.
4. Load manifests explicitly and fail fast. `register_manifest_dir` reads a directory the operator names on the command line (`serve --manifests DIR`), sorts paths for determinism, and aborts startup on the first invalid or conflicting manifest. Nothing is fetched or loaded at runtime; registration remains the reviewed, build/startup-time act.
5. Add a real JSON Schema (`schemas/device-manifest.schema.json`, draft 2020-12) for editors and CI, while keeping the Rust interpreter authoritative for rules JSON Schema cannot express (overlap, capability consistency, effect mapping).
6. Provide fictitious examples only: `examples/declarative-driver/manifest.toml` (synthetic `0xF0F0:0x0001`) and the compiled `examples/native-driver` crate (`0xF0F1:0x0002`). They carry no hardware-support claim and are not registered in the shipped runtime. Real device entries under `devices/` still require authorized protocol evidence.

## Consequences

Manifests are testable without hardware: encoding, capability gating, rate limiting, identification and the daemon request flow are covered by unit and integration tests over an in-memory transport. The report budget is small and explicit, and unsupported or malformed requests return `NotSupported`/structured faults rather than simulated success. Because the interpreter only models volatile lighting, richer protocols (displays, macros, feature reads, firmware) require a reviewed native crate or a future contract revision rather than a manifest. DRA, packaging/release and any hardware operation remain explicitly out of scope. See the [driver system](../driver-system.md), [SDK reference](../sdk.md), [daemon guide](../daemon.md) and the [status ledger](../implementation-status.md).
