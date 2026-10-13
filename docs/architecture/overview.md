# Architecture overview

**Status: Phase 1 Rust foundation implemented; application runtime remains planned.** This document follows the [master specification](../../OPENCODE_IMPLEMENTATION_MASTER.md). The [SDK reference](sdk.md) describes actual contracts; the [status ledger](implementation-status.md) records validation.

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
| `crates/core`, `crates/device-sdk`, `crates/device-registry` | Inventory, versioned contracts, explicit factory matching | Implemented libraries with unit tests |
| Other `crates/` directories | Transports, domains, IPC, storage, research | Directories only |
| `devices/` | Community evidence and implemented support | Empty manufacturer/model placeholders |
| `schemas/` | Versioned interchange validation | Empty JSON files |
| `examples/`, `tools/`, `tests/` | Future real examples, utilities, and tests | Directories only |
| `.agents/skills/` | Seven AI procedures | Documentary foundation provided |
| `docs/` | Architecture, contributor and research guidance | Foundation documents; not a software manual |
| `.github/` | Future templates, CI, and ownership | Empty placeholders; no active CI |
| `config/agents/` | Optional REA/MCP example | Empty placeholder; not usable configuration |

Cargo membership follows only the three implemented crates. Rust 1.99.0 is pinned and `Cargo.lock` is generated. Node/application manifests remain empty. Existing placeholders and all pre-existing directories are preserved.

## Contract requirements

SDK 1.0 implements observed `DeviceDescriptor` fields (VID/PID, manufacturer/product, optional serial, path/interface, usage page/usage, interface number, transport, and known firmware); model-level `DeviceId`, physical `DeviceInstanceId`, and endpoint `DeviceInterfaceId`; granular `Capabilities`; driver/factory traits; and bounded `Transport` contracts for HID input/output/feature reports. It supplies no physical transport or driver implementation.

`DriverRegistry` considers interfaces/usage and optional exact product/firmware evidence rather than VID/PID alone; ambiguity is an error. `DeviceError` implements `NotSupported`, `NotConnected`, `PermissionDenied`, `InvalidArgument`, `Timeout`, `TransportError`, `ProtocolError`, and `IncompatibleVersion`. Registry/Core errors preserve typed sources. Core commits caller-provided inventory snapshots atomically; it does not monitor OS connections.

The SDK has explicit contract version 1.0 and typed serde DTOs with semantic validators. Future IPC, profiles, research reports, and manifest models still need independent versions and compatibility policies. SQLite changes require migrations; TOML needs typed parsing/semantic validation plus interchange schemas. No manifest/profile/research schema is implemented in Phase 1.

## Domain scope

Lighting depends on device-specific zones, effects, and limits. Displays require evidenced resolution, format, and rate constraints. Initial macros are edited, validated, persisted, and safely simulated, without global input injection. Profiles use local state and versioned JSON exports. Storage must not collect vendor binaries or raw sensitive captures.

## Evidence and decisions

Support claims must remain capability-specific and use `simulated`, `inferred`, `tested`, or `validated-on-hardware` with provenance and limits. No hardware has been validated here.

See [device lifecycle](device-lifecycle.md), [driver system](driver-system.md), [IPC](ipc.md), [security](security.md), [ADR 0001](decisions/0001-foundation-boundaries.md), and [ADR 0002](decisions/0002-sdk-and-registry-contracts.md).
