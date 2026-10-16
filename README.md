# Peripheral Hub

**A community-driven project for open, capability-aware peripheral control.**

[Português brasileiro](README.pt-BR.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [Implementation status](docs/architecture/implementation-status.md)

## Project status

Peripheral Hub has an implemented **Rust foundation and daemon**: SDK 1.0, Registry, Core, real `hidapi` discovery/report transport, versioned local IPC, connection management, explicit software simulation, and SQLite-backed profile persistence with typed profile IPC. The Rust daemon and diagnostic client run independently of a GUI. Desktop, physical device drivers, profile import/export, DRA, interchange schemas and CI remain unimplemented; no physical control capability is validated.

**No device compatibility or physical functionality is currently claimed.** The existing `devices/manufacturer/device-model/` directory is a structural placeholder, not a supported product. The project specification is [OPENCODE_IMPLEMENTATION_MASTER.md](OPENCODE_IMPLEMENTATION_MASTER.md); the [status ledger](docs/architecture/implementation-status.md) distinguishes delivered documentation from future software milestones.

## Why Peripheral Hub?

Peripheral configuration is often tied to separate proprietary vendor applications. Peripheral Hub aims to provide one extensible desktop experience for keyboards, mice, and eventually other peripherals, with transparent support levels and community-maintained protocol documentation.

The guiding principle is simple: **show only capabilities actually supported and individually validated for the connected device**. Partial support is useful; an inferred protocol is not proof that a feature works on hardware.

## Planned functionality

- HID discovery and identification, including composite devices and multiple interfaces.
- RGB colors, brightness, zones, and effects where supported.
- Integrated display control where the protocol has been validated.
- Button remapping and macros when technically supported; the initial macro scope is editing, validation, persistence, and safe simulation.
- Persistent profiles, with versioned JSON import/export.
- A community device catalog with capability-level evidence and validation records.
- A standalone DRA for local research sessions, capture import, packet comparison, sanitized reports, and evidence-based driver scaffolding.

These are roadmap items, not currently available features. Firmware flashing, bootloader operations, unknown persistent writes, and arbitrary native plugin loading are outside the MVP scope.

## Planned technology

| Area | Direction required by the specification |
| --- | --- |
| Application and workspaces | Stable Rust, Cargo workspace; pnpm for JavaScript packages |
| Desktop | Tauri 2, React, TypeScript, Vite |
| Device access | `hidapi` behind a project-owned transport abstraction |
| Async processing | Tokio where asynchronous execution adds value |
| Local communication | Versioned, same-user IPC; Windows named pipes and protected Unix domain sockets |
| Persistence | SQLite application state; versioned JSON profiles |
| Drivers | Validated declarative TOML or reviewed Rust crates compiled into the application |
| Interchange validation | Typed models, semantic checks, and JSON Schema |
| Research | Rust DRA; optional REA, USBPcap, Wireshark/TShark |

Windows 10/11 is the first product target. The Phase 1 libraries were checked/tested on a Linux laboratory host; this does not establish Linux desktop or hardware support. Windows/macOS builds and future OS backends remain unvalidated. Python may be used for justified laboratory utilities, never as an application runtime.

## Architectural direction

```text
React/TypeScript → restricted Tauri API → typed IPC client
    → user-session Rust daemon → core / registry / SDK / drivers
    → HID or USB transport → peripheral

Standalone DRA → evidence / sanitized fixtures / reports
    → reviewed device contribution (outside the normal runtime)
```

The frontend must not access USB/HID directly. Drivers declare granular capabilities, unsupported operations return explicit errors, and declarative input is treated as untrusted. See the [architecture overview](docs/architecture/overview.md).

## Getting started at this stage

Install Rust with rustup and an OS-native linker. `rust-toolchain.toml` pins stable Rust **1.99.0**, rustfmt, and Clippy. From the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo build --workspace --locked
```

Run the explicit software-only backend in one terminal and query it from another:

```sh
cargo run -p peripheral-hub-daemon -- serve --simulate
cargo run -p peripheral-hub-daemon -- health
cargo run -p peripheral-hub-daemon -- list
```

For non-destructive physical HID inventory, use `cargo run -p peripheral-hub-daemon -- enumerate`; physical daemon mode is `serve` without `--simulate`. Unknown devices are never automatically opened or written.

The desktop frontend (React/TypeScript) is a pnpm workspace member. Install Node 20+/pnpm, then:

```sh
pnpm install
pnpm --filter @peripheral-hub/desktop typecheck
pnpm --filter @peripheral-hub/desktop test
pnpm --filter @peripheral-hub/desktop build
```

It expects a running daemon and reaches it only through the restricted Tauri `daemon_call` bridge. The Tauri shell in `apps/desktop/src-tauri` is real source but is not built or launched here because platform webview dependencies are absent, so there is no desktop installer. See the [daemon and transport guide](docs/architecture/daemon.md), [desktop reference](docs/architecture/desktop.md), [SDK contracts](docs/architecture/sdk.md), and [CONTRIBUTING.md](CONTRIBUTING.md). Linux daemon tests and Windows cross-target static checks are recorded separately; Windows runtime/hardware control is unvalidated.

## Community contributions

Contributors can help with documentation, accessibility, architecture review, safe protocol investigation, and eventually driver and feature development. Device contributions will need interface identity, protocol evidence, sanitized fixtures, negative-path tests, and a hardware validation matrix. Owning a vendor installer alone does not establish the full device protocol.

AI assistance is optional. The [seven repository skills](.agents/skills/README.md) guide evidence collection, implementation, validation, and review; humans remain responsible for authorization and review. Do not upload proprietary binaries, raw USB captures, personal data, or credentials to issues, pull requests, or external AI services.

See the bilingual [device contribution guide](docs/en/contributors/adding-a-device.md) and [security policy](SECURITY.md).

## License and governance

The existing GitHub repository includes the Apache-2.0 text in [LICENSE](LICENSE), preserved from its initial commit. The master specification still requires explicit maintainer confirmation before treating that choice as definitive for official publication; that confirmation remains pending.

Maintainer identities, security contact details, community conduct enforcement, and repository ownership assignments remain pending; see the status ledger. No maintainer accounts or contact addresses are invented in this documentation.
