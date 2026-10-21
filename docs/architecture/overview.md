# Architecture overview

**Status: SDK foundation, HID/IPC/daemon, SQLite profile/macro persistence and the desktop frontend/bridge are implemented as software; the Tauri shell is not built or run here, DRA/research tooling and physical drivers remain unimplemented.** This document follows the [master specification](../../OPENCODE_IMPLEMENTATION_MASTER.md). See [SDK reference](sdk.md), [daemon guide](daemon.md), [desktop reference](desktop.md), [ADR 0004](decisions/0004-storage-and-profiles.md), [ADR 0005](decisions/0005-desktop-ui.md) and [validation ledger](implementation-status.md).

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

apps/device-research-assistant  standalone research CLI (planned placeholder)
          → local evidence → sanitized report / fixture → reviewed contribution
```

The frontend must not access USB/HID or submit arbitrary packets. The daemon owns device state, lifecycle, permission/error handling, and operation routing. The planned DRA is not part of the shipped application path, and optional REA is not a runtime dependency.

## Existing repository responsibilities

| Path | Planned responsibility | Current state |
| --- | --- | --- |
| `apps/desktop/` | Tauri 2 + React/TypeScript/Vite interface | React UI, restricted Tauri bridge source and pnpm workspace; shell not built here (webview deps absent) |
| `apps/daemon/` | User-session runtime, lifecycle, simulation and IPC diagnostics | Implemented binary/library and software integration tests |
| `apps/device-research-assistant/` | Rust research CLI | Empty manifest/source placeholder; not a workspace member |
| `crates/core`, `crates/device-sdk`, `crates/device-registry` | Inventory, versioned contracts, explicit factory matching | Implemented libraries with unit tests |
| `crates/transport-hid`, `crates/ipc` | Real hidapi discovery/report transport and protected local IPC | Implemented, with Linux software/runtime and Windows static evidence distinguished |
| `crates/storage`, `crates/profiles` | SQLite connection/migrations and versioned profile persistence | Implemented libraries with unit tests; profile IPC wired into the daemon |
| Other `crates/` directories | Transports, domains, research, desktop bridges | Directories only |
| `devices/` | Community evidence and implemented support | Empty manufacturer/model placeholders |
| `schemas/` | Versioned interchange validation | `device-manifest.schema.json` implemented; profile and research schema files remain empty; Rust typed validation is authoritative |
| `examples/`, `tools/`, `tests/` | Future real examples, utilities, and tests | Directories only |
| `.agents/skills/` | Seven AI procedures | Documentary foundation provided |
| `docs/` | Architecture, contributor and research guidance | Foundation documents; not a software manual |
| `.github/` | Future templates, CI, and ownership | Empty placeholders; no active CI |
| `config/agents/` | Optional REA/MCP example | Empty placeholder; not usable configuration |

Cargo membership follows eleven implemented members: SDK, Registry, Core, HID transport, IPC, storage, profiles, macros, the bounded declarative driver interpreter, a fictitious native example and the daemon app; the Tauri shell and the empty DRA placeholder are excluded. Rust 1.99.0 is pinned and `Cargo.lock` is generated. The pnpm workspace builds the desktop frontend. `schemas/device-manifest.schema.json` is implemented; the other schema files remain empty. Existing organization is preserved; SDK/Core/Registry sources remain unchanged.

## Contract requirements

SDK 1.0 implements observed identity, capabilities, driver/factory and bounded report contracts. Its own crate remains transport-independent; Phase 2 supplies hidapi implementation externally. Discovery uses conservative session-epoch unit/endpoint tokens and does not infer composite grouping across paths. No proprietary physical driver is bundled.

`DriverRegistry` considers interfaces/usage and optional exact product/firmware evidence rather than VID/PID alone; ambiguity is an error. `DeviceError` implements `NotSupported`, `NotConnected`, `PermissionDenied`, `InvalidArgument`, `Timeout`, `TransportError`, `ProtocolError`, and `IncompatibleVersion`. Registry/Core errors preserve typed sources. Core commits caller-provided inventory snapshots atomically; it does not monitor OS connections.

The SDK has explicit contract version 1.0 and typed serde DTOs with semantic validators. IPC (version 5), profiles, macros and declarative manifests carry independent versions and compatibility policies. SQLite changes require migrations; TOML is parsed and semantically validated. DRA research contracts remain unimplemented.

## Domain scope

Lighting depends on device-specific zones, effects, and limits. Displays require evidenced resolution, format, and rate constraints. Initial macros are edited, validated, persisted, and safely simulated, without global input injection. Profiles use local state and versioned JSON exports. Storage must not collect vendor binaries or raw sensitive captures.

## Evidence and decisions

Support claims must remain capability-specific and use `simulated`, `inferred`, `tested`, or `validated-on-hardware` with provenance and limits. No hardware has been validated here.

See [device lifecycle](device-lifecycle.md), [driver system](driver-system.md), [IPC](ipc.md), [security](security.md), [ADR 0001](decisions/0001-foundation-boundaries.md), [ADR 0002](decisions/0002-sdk-and-registry-contracts.md), [ADR 0009](decisions/0009-macro-persistence.md) and [ADR 0010](decisions/0010-display-contract-preview.md).
