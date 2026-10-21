# Architecture overview

**Status: SDK foundation, HID/IPC/daemon, SQLite profile/macro persistence, the desktop frontend/bridge and the standalone DRA MVP are implemented as software; the Tauri shell is not built or run here, the DRA is hardware-free/synthetic-only, and physical drivers remain unimplemented.** This document follows the [master specification](../../OPENCODE_IMPLEMENTATION_MASTER.md). See [SDK reference](sdk.md), [daemon guide](daemon.md), [desktop reference](desktop.md), [ADR 0004](decisions/0004-storage-and-profiles.md), [ADR 0005](decisions/0005-desktop-ui.md) and [validation ledger](implementation-status.md).

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

apps/device-research-assistant  standalone research CLI (implemented MVP, hardware-free)
          → local evidence → sanitized report / fixture → reviewed contribution
```

The frontend must not access USB/HID or submit arbitrary packets. The daemon owns device state, lifecycle, permission/error handling, and operation routing. The DRA is not part of the shipped application path, and optional REA is not a runtime dependency.

## Existing repository responsibilities

| Path | Planned responsibility | Current state |
| --- | --- | --- |
| `apps/desktop/` | Tauri 2 + React/TypeScript/Vite interface | React UI, restricted Tauri bridge source and pnpm workspace; shell not built here (webview deps absent) |
| `apps/daemon/` | User-session runtime, lifecycle, simulation and IPC diagnostics | Implemented binary/library and software integration tests |
| `apps/device-research-assistant/` | Rust research CLI | Implemented standalone library/binary with 8 synthetic integration tests; local JSON sessions, bounded import, analysis, reports and commented scaffold; no pcap parser or hardware write path |
| `crates/core`, `crates/device-sdk`, `crates/device-registry` | Inventory, versioned contracts, explicit factory matching | Implemented libraries with unit tests |
| `crates/transport-hid`, `crates/ipc` | Real hidapi discovery/report transport and protected local IPC | Implemented, with Linux software/runtime and Windows static evidence distinguished |
| `crates/storage`, `crates/profiles` | SQLite connection/migrations and versioned profile persistence | Implemented libraries with unit tests; profile IPC wired into the daemon |
| Other `crates/` directories | Transports, domains, research, desktop bridges | Directories only |
| `devices/` | Community evidence and implemented support | Empty manufacturer/model placeholders |
| `schemas/` | Versioned interchange validation | Device manifest, profile, research capture and research report schemas documented; Rust semantic validation is authoritative; other schema files remain empty |
| `examples/`, `tools/`, `tests/` | Future real examples, utilities, and tests | Directories only |
| `.agents/skills/` | Seven AI procedures | Documentary foundation provided |
| `docs/` | Architecture, contributor and research guidance | Foundation documents; not a software manual |
| `.github/` | Future templates, CI, and ownership | Empty placeholders; no active CI |
| `config/agents/` | Optional REA/MCP example | Empty placeholder; not usable configuration |

Cargo membership follows twelve implemented members: SDK, Registry, Core, HID transport, IPC, storage, profiles, macros, the bounded declarative driver interpreter, a fictitious native example, the daemon app and the standalone `device-research-assistant`; the Tauri shell is a standalone manifest excluded from the workspace. Rust 1.99.0 is pinned and `Cargo.lock` is generated. The pnpm workspace builds the desktop frontend. The device-manifest, profile, research-capture and research-report schema files are documented; the remaining interchange schema files are empty. Rust typed/semantic validation is authoritative, and no external JSON Schema validator is claimed. Existing organization is preserved; SDK/Core/Registry sources remain unchanged.

## Contract requirements

SDK 1.0 implements observed identity, capabilities, driver/factory and bounded report contracts. Its own crate remains transport-independent; Phase 2 supplies hidapi implementation externally. Discovery uses conservative session-epoch unit/endpoint tokens and does not infer composite grouping across paths. No proprietary physical driver is bundled.

`DriverRegistry` considers interfaces/usage and optional exact product/firmware evidence rather than VID/PID alone; ambiguity is an error. `DeviceError` implements `NotSupported`, `NotConnected`, `PermissionDenied`, `InvalidArgument`, `Timeout`, `TransportError`, `ProtocolError`, and `IncompatibleVersion`. Registry/Core errors preserve typed sources. Core commits caller-provided inventory snapshots atomically; it does not monitor OS connections.

The SDK has explicit contract version 1.0 and typed serde DTOs with semantic validators. IPC, profiles, macros, declarative manifests and DRA research data carry independent versions and compatibility policies. SQLite changes require migrations; TOML is parsed and semantically validated. The DRA capture/session/analysis/report versions are intentionally separate from SDK 1.0 and IPC 5.

## Domain scope

Lighting depends on device-specific zones, effects, and limits. Displays require evidenced resolution, format, and rate constraints. Initial macros are edited, validated, persisted, and safely simulated, without global input injection. Profiles use local state and versioned JSON exports. Storage must not collect vendor binaries or raw sensitive captures.

## Evidence and decisions

Support claims must remain capability-specific and use `simulated`, `inferred`, `tested`, or `validated-on-hardware` with provenance and limits. No hardware has been validated here.

See [device lifecycle](device-lifecycle.md), [driver system](driver-system.md), [IPC](ipc.md), [security](security.md), [DRA reference](dra.md), [ADR 0001](decisions/0001-foundation-boundaries.md), [ADR 0002](decisions/0002-sdk-and-registry-contracts.md) and [ADR 0011](decisions/0011-dra-mvp.md).
