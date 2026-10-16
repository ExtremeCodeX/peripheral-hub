# Device SDK 1.0 and Rust foundation

## Implemented packages and boundaries

The preserved Phase 1 foundation contains three library crates (Cargo package version `0.1.0`):

- `peripheral-hub-device-sdk`: identity, typed capabilities/commands/errors, matching rules, driver/factory and transport contracts, semantic validators, and a transport bounds wrapper.
- `peripheral-hub-device-registry`: explicit factory registration, metadata validation, unique interface matching, and driver construction without I/O.
- `peripheral-hub-core`: in-memory inventory replacement and grouping over caller-supplied descriptors and a Registry.

Dependency direction is **Core → Registry → SDK**, with Core also using SDK types directly. Drivers implement `DriverFactory` and `DeviceDriver`, not Core internals. Phase 2 adds HID, IPC and daemon packages, documented in the [daemon guide](daemon.md), without changing these contracts. The declarative interpreter (`peripheral-hub-driver-declarative`) and a fictitious native example (`examples/native-driver`) are built on these same contracts; the daemon loads declarative manifests only through `serve --manifests DIR`. The physical registry remains empty; only explicit simulation registers a fictitious factory. The desktop frontend exists under `apps/desktop` and consumes these contracts only through the daemon's local IPC; no proprietary physical driver exists. Profile persistence lives in the separate storage/profiles layer, not in the SDK.

## Version policy

`SDK_CONTRACT_VERSION` is `ContractVersion { major: 1, minor: 0 }`, independent of Cargo versions, future IPC versions, profile formats, and manifest schemas. Breaking type/semantic changes require a major change; additive changes require a minor change and compatibility review. Only the same major and a minor no newer than this SDK are accepted by `ensure_supported()` and Registry registration.

`Versioned<T>` supplies an explicit SDK envelope; after deserialization, call `into_supported()` before use and validate the payload separately. Unknown versions are rejected, not silently interpreted. SDK JSON types reject unknown fields and enum variants. An older consumer therefore rejects newer DTO shapes even if their version is mislabeled; it does not promise forward compatibility. No IPC wire framing or manifest/profile schema is defined by these types.

## Identity and matching

`DeviceId` identifies a model/factory. `DeviceInstanceId` is an opaque physical-unit token supplied by future discovery. `DeviceInterfaceId` is a distinct opaque endpoint token; composite devices can share a physical instance across interfaces, including HID/non-HID interfaces. Tokens accept 1..128 ASCII bytes of alphanumerics and `-_.:`; constructors and serde reject invalid tokens.

`DeviceDescriptor` retains observed VID/PID, transport, raw OS locator bytes (non-UTF-8-safe), optional interface/usage, manufacturer/product, serial, and known firmware. `validate()` bounds paths and text and rejects HID usage on non-HID interfaces. Unknown values remain `None`. Physical grouping is supplied by discovery and cannot be derived from VID/PID alone; tokens are session-local, not promises of identity across reboot.

Each `DeviceMatchRule` is a conjunction of exact constraints. At least an interface number or HID usage pair is required beyond VID/PID. Product and firmware constraints are optional exact additional evidence; missing required observations fail the match. Multiple rules in one model are alternatives and count once. Multiple matching models yield `RegistryError::AmbiguousMatch`; registration order/specificity never silently selects a winner.

## Capability and command semantics

`Capabilities::default()` declares no support. Optional lighting, display, button, macro, and profile declarations have semantic bounds. Lighting has explicit unique zones, brightness support, and effects; displays require dimensions/formats/rate/frame byte bounds; remaining declarations specify bounded counts/delays.

SDK 1.0 has one typed device command: `DeviceCommand::SetLighting`. `Capabilities::validate_command()` checks zone/effect/brightness support and bounds. It does not execute lighting. Display/button/macro/profile types are declarations, not implemented domain mechanisms. There are no raw packet, firmware, persistent-write, input-injection, or arbitrary execution commands.

`DeviceDriver` is synchronous and `Send`: `open`, `close`, `capabilities`, `read_state`, and `execute`. State queries and commands default to `NotSupported`; connection lifecycle must be supplied truthfully by later implementations. `DriverFactory: Send + Sync` exposes validated `DriverMetadata` and constructs a selected driver without hardware I/O. Native factories are explicitly registered reviewed code; runtime loading is absent.

`CommandOutcome` and `DeviceState` identify `ExecutionMode::Simulated` or `Physical`. Physical describes an operation's origin, not certification. Hardware validation still needs capability/firmware/platform evidence. The SDK contracts cannot independently verify a reviewed driver's claims or packet safety.

## Transport contract only

`Transport` separates HID input, output, and feature report operations. `HidReport` normalizes report ID separately from payload, including ID zero. Byte counts and limits exclude the report ID; future `hidapi` adapters must translate their OS conventions. Reads specify an expected report ID. Output reads and input writes are invalid.

`TransportLimits` caps each payload at 65535 bytes and the nonzero maximum timeout at 30 seconds; a zero report byte limit means `NotSupported`. `CheckedTransport<T>` snapshots validated limits, rejects invalid operations/timeouts/lengths before delegation, checks returned read ID/size, rejects incomplete/invalid write counts, and preserves backend errors. It cannot enforce elapsed time or allocation limits inside an untrusted backend: the reviewed backend must honor deadlines and bounds. Rate limits and safe protocol operations remain future backend/driver responsibilities.

The SDK itself provides no OS backend. Phase 2 implements a real reviewed/bounded `hidapi` adapter and explicit simulation in `peripheral-hub-transport-hid`. OS cancellation/deadline limits are documented in the daemon guide. The original private SDK doubles remain contract-test evidence, not physical compatibility evidence.

## Core inventory and errors

`PeripheralCore` owns an immutable registered set. `replace_inventory()` accepts a complete descriptor snapshot, validates/matches all entries, and commits atomically. Invalid descriptors, duplicate endpoint tokens/locators, inconsistent unit VID/PID, or matching ambiguity preserve the previous inventory. `InventoryChange` counts added/removed/updated interfaces between supplied snapshots; this is not real hotplug detection.

`Identification::Compatible(model)` means a unique factory matches; it does not mean a transport is open, a device command succeeded, or physical support was validated. `Unknown` keeps unsupported interfaces visible. Instance-based lookup groups interfaces without collapsing devices with the same VID/PID.

`DeviceError` includes explicit unsupported, disconnected, denied, invalid argument, timeout, transport, protocol, and incompatible-version cases. Registry/Core errors retain typed sources. Libraries emit structured `tracing` events with stable event names, counts, public model tokens, and static error codes; they do not install a global subscriber. Applications will choose sinks later. Default descriptor/report Debug omits serials, paths, instance tokens, and payload bytes. DTO serialization still contains local data and must never be treated as a sanitized public export.

## Build and test

Install [Rust using rustup](https://www.rust-lang.org/tools/install) and an OS-native linker (Linux C toolchain, Windows MSVC Build Tools with its SDK, or macOS Command Line Tools). `rust-toolchain.toml` pins stable release `1.99.0` with rustfmt/Clippy; no earlier Rust version is claimed. From the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo build --workspace --locked
cargo doc --workspace --no-deps
```

Tests use synthetic descriptors only. Current validation was performed on a Linux laboratory host; Windows/macOS builds and physical compatibility are unverified. See the [status ledger](implementation-status.md) for exact commands, temporary toolchain/linker adaptations, failures corrected, and results. Network access is needed for first-time toolchain/crate downloads unless already cached; preserve the generated `Cargo.lock`.
