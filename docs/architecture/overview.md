# Architecture overview

**Status: required design, not implemented.** This foundation documents the [master specification](../../OPENCODE_IMPLEMENTATION_MASTER.md). The [status ledger](implementation-status.md) is the source for actual delivery.

## Layers and trust boundaries

```text
apps/desktop/src                 React + TypeScript presentation
          ↓ restricted Tauri commands / DTOs
apps/desktop/src-tauri           narrow native desktop bridge
          ↓ typed, versioned same-user local IPC
apps/daemon                     user-session orchestration
          ↓ core / profiles / storage / device registry
crates/device-sdk               identity, capabilities, driver and transport contracts
          ↓ reviewed native driver / bounded declarative interpreter
crates/transport-hid|usb         operating-system device access
          ↓
physical peripheral             independently validated capabilities

apps/device-research-assistant  standalone research CLI (planned)
          → local evidence → sanitized report / fixture → reviewed contribution
```

The frontend must not access USB/HID or submit arbitrary packets. The daemon owns device state, lifecycle, permission/error handling, and operation routing. The DRA is not part of the shipped application path, and optional REA is not a runtime dependency.

## Existing repository responsibilities

| Path | Planned responsibility | Current state |
| --- | --- | --- |
| `apps/desktop/` | Tauri 2 + React/TypeScript/Vite interface | Empty package/config placeholders and directories |
| `apps/daemon/` | User-session runtime and IPC server | Empty manifest and source directory |
| `apps/device-research-assistant/` | Rust research CLI | Empty manifest/source placeholder |
| `crates/` | Core, SDK, registry, transports, domains, IPC, storage, research | Directories only |
| `devices/` | Community evidence and implemented support | Empty manufacturer/model placeholders |
| `schemas/` | Versioned interchange validation | Empty JSON files |
| `examples/`, `tools/`, `tests/` | Future real examples, utilities, and tests | Directories only |
| `.agents/skills/` | Seven AI procedures | Documentary foundation provided |
| `docs/` | Architecture, contributor and research guidance | Foundation documents; not a software manual |
| `.github/` | Future templates, CI, and ownership | Empty placeholders; no active CI |
| `config/agents/` | Optional REA/MCP example | Empty placeholder; not usable configuration |

Cargo membership must follow implemented crates, not directory names. Node/Rust manifests and lockfiles are not yet valid workspaces. Existing placeholders and all existing directories are preserved.

## Contract requirements

The future SDK needs observed `DeviceDescriptor` fields (VID/PID, manufacturer/product, optional serial, path/interface, usage page/usage, interface number, transport, and known firmware); model-level `DeviceId` and physical `DeviceInstanceId`; granular `Capabilities`; driver lifecycle/operation traits; and testable `Transport` abstractions for HID input/output/feature reports with bounds.

`DriverRegistry` must consider interfaces, usage, revision evidence, and ambiguity rather than VID/PID alone. Typed `DeviceError` cases must include `NotSupported`, `NotConnected`, `PermissionDenied`, `InvalidArgument`, `Timeout`, `TransportError`, and `ProtocolError`. Failure must never become fictitious success. These names are design requirements, not Rust types currently available to import.

Contracts, IPC, profiles, research reports, and manifest models need explicit versions and documented compatibility policies. SQLite changes require migrations; TOML requires typed parsing and semantic validation as well as interchange schemas. No schemas or DTOs are fabricated in this stage.

## Domain scope

Lighting depends on device-specific zones, effects, and limits. Displays require evidenced resolution, format, and rate constraints. Initial macros are edited, validated, persisted, and safely simulated, without global input injection. Profiles use local state and versioned JSON exports. Storage must not collect vendor binaries or raw sensitive captures.

## Evidence and decisions

Support claims must remain capability-specific and use `simulated`, `inferred`, `tested`, or `validated-on-hardware` with provenance and limits. No hardware has been validated here.

See [device lifecycle](device-lifecycle.md), [driver system](driver-system.md), [IPC](ipc.md), [security](security.md), and [ADR 0001](decisions/0001-foundation-boundaries.md).
