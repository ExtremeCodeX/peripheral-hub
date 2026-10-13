# Peripheral Hub

**A community-driven project for open, capability-aware peripheral control.**

[Português brasileiro](README.pt-BR.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [Implementation status](docs/architecture/implementation-status.md)

## Project status

Peripheral Hub is in its **documentation and organizational foundation stage**. The repository contains a planned directory structure, contributor guidance, architectural rules, and AI skill procedures. Application source files, package manifests, schemas, and CI files are still empty placeholders. There is no runnable desktop application, daemon, Device Research Assistant (DRA), or validated hardware driver yet.

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

Windows 10/11 is the first target. Linux and macOS are future targets with separated transport backends, not platforms currently tested or supported. Python may be used for justified laboratory utilities, never as an application runtime.

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

Start with [CONTRIBUTING.md](CONTRIBUTING.md), [AGENTS.md](AGENTS.md), and the status ledger. Documentation contributions and authorized, local protocol research can begin now.

There are **no working installation, application launch, build, or test commands yet**: the Cargo and pnpm manifests and lockfiles have no content. The contributor guide records future validation gates without presenting them as runnable instructions. Rust/Node versions, platform prerequisites, and exact daemon/desktop/DRA commands must be documented when real workspaces exist.

## Community contributions

Contributors can help with documentation, accessibility, architecture review, safe protocol investigation, and eventually driver and feature development. Device contributions will need interface identity, protocol evidence, sanitized fixtures, negative-path tests, and a hardware validation matrix. Owning a vendor installer alone does not establish the full device protocol.

AI assistance is optional. The [seven repository skills](.agents/skills/README.md) guide evidence collection, implementation, validation, and review; humans remain responsible for authorization and review. Do not upload proprietary binaries, raw USB captures, personal data, or credentials to issues, pull requests, or external AI services.

See the bilingual [device contribution guide](docs/en/contributors/adding-a-device.md) and [security policy](SECURITY.md).

## License and governance

Apache-2.0 is **proposed, pending maintainer confirmation**. The existing `LICENSE` file is empty and does not grant a license. Do not represent the project as already Apache-2.0 licensed or publish an official release before that decision.

Maintainer identities, security contact details, community conduct enforcement, and repository ownership assignments remain pending; see the status ledger. No maintainer accounts or contact addresses are invented in this documentation.
