# Contributing to Peripheral Hub

[Português brasileiro](docs/pt-BR/contributors/contributing.md)

Thank you for helping build a transparent, community-maintained peripheral platform. The [master specification](OPENCODE_IMPLEMENTATION_MASTER.md) defines the product; [AGENTS.md](AGENTS.md) and the [architecture overview](docs/architecture/overview.md) define contribution boundaries.

## What contributors can do now

Phase 2 builds on the unchanged Rust SDK, Registry and Core with real HID discovery/transport, versioned protected IPC, a daemon and explicit simulation. Read [SDK contracts](docs/architecture/sdk.md) and the [daemon guide](docs/architecture/daemon.md) for actual APIs/commands. Desktop, physical drivers, profiles, DRA, manifest validators, schemas and CI remain future work. This execution stops before Phase 3 desktop implementation.

The existing repository includes Apache-2.0 text in `LICENSE`. Preserve it; explicit maintainer confirmation required by the master specification remains pending before official distribution. A published security contact and community conduct/enforcement policy are also pending. Be respectful, constructive, and mindful of personal information in every interaction.

## Development environment

The intended toolchain is stable Rust with Cargo, Node.js compatible with the eventual frontend dependencies, and pnpm. Windows 10/11 is the initial target; Linux/macOS portability must be preserved. Desktop system prerequisites should be taken from [Tauri 2 documentation](https://v2.tauri.app/start/prerequisites/) and recorded for the implemented target.

Rust is pinned to stable 1.99.0 with rustfmt/Clippy, edition 2024, and a generated `Cargo.lock`. Install rustup plus a native linker; six real packages are workspace members. Run `cargo run -p peripheral-hub-daemon -- serve --simulate` for software-only development and use `health`/`list` in another terminal. Physical `enumerate` is non-destructive; no physical driver is registered. pnpm/DRA manifests remain empty. Preserve real lockfiles and add members only with actual source; do not commit dependency trees or build output.

The canonical repository supplied by the maintainer is [ExtremeCodeX/peripheral-hub](https://github.com/ExtremeCodeX/peripheral-hub), with default branch `main`. Fork it, clone your fork, and create a focused branch. The local checkout now contains Git history and an `origin` remote; the original documentary audit predated that setup.

## Adding a device

Use the [device contribution guide](docs/en/contributors/adding-a-device.md), with or without an AI agent.

1. Record observed VID/PID, interface, usage page/usage, transport, OS, firmware/revision, and distinguishing identifiers. Sanitize serials and private paths.
2. Obtain authorized evidence. Vendor executables can inform hypotheses; controlled captures and physical tests establish behavior. REA is optional, and DRA is planned but not yet runnable.
3. Document report IDs, lengths, byte layouts, initialization, ACKs, checksums, timing, safe command bounds, uncertainty, and provenance. Never replay unknown packets or assert capabilities from VID/PID alone.
4. Once SDK/validators exist, choose a bounded declarative driver for simple modeled commands or a reviewed, compiled Rust driver for complex protocols. Implement only evidenced capabilities.
5. Add software tests with synthetic/sanitized fixtures and opt-in hardware validation. Mark untested firmware, platforms, and capabilities explicitly.
6. Submit the artifact set under `devices/<manufacturer>/<model>/`: `manifest.toml`, `protocol.md`, `validation.md`, `tests/`, and `fixtures/`; add `Cargo.toml` + `src/` only for an implemented native driver. Future fictitious examples belong in `examples/`, never in the validated device catalog.

Raw `.exe`/DLL/firmware files and full captures stay in ignored local storage. Only necessary, authorized, sanitized evidence may enter a PR. Ignore rules are a convenience, not a substitute for reviewing the files to be committed.

## Developing features

Agree on a focused scope and acceptance criteria, identify the owning crate or desktop feature, and document significant changes with an ADR. Preserve the frontend/daemon/transport boundary, same-user versioned IPC, explicit unsupported errors, and granular capabilities. Do not add runtime native plugins, an elevated daemon, arbitrary frontend commands, or Python as an application runtime.

For frontend work, cover loading, empty, disconnected, denied, failed, simulated, and unsupported states and accessible navigation. For macros, limit the initial work to editing, validation, persistence, and safe simulation. No global input injection without a separately authorized, tested stage. For storage, define migrations and schema versions rather than silently changing persisted formats.

## Tests and review gates

For the implemented Rust libraries, run the following gates from the root. For documentation changes, also inspect content, relative links, frontmatter, scope, translations, and status claims.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo build --workspace --locked
```

Future frontend work must run `pnpm install --frozen-lockfile` and only defined lint/test/build scripts; future manifest/schema validation needs real validators. Those commands are unavailable for the current placeholders. Report failures and unavailable tools honestly.

Meaningful tests should cover typed contracts, ambiguous driver matching, lengths/limits, invalid inputs, `NotSupported`, disconnects, timeouts, IPC authorization, profile migrations, and DRA correlation/sanitization. Frontend tests must distinguish simulated and physical states. Hardware tests are opt-in, outside automatic CI, with exact OS/device/firmware/capability evidence.

## Using AI skills

Read `AGENTS.md` first and load the relevant [skill](.agents/skills/README.md). The device flow is `add-device` → `reverse-engineer`/`capture-protocol` → `develop-driver` → `validate-device` → `review-contribution`; use `develop-feature` for application work. This is a coordinated procedure, not mandatory agent automation. The same evidence and review requirements apply to human-only contributions.

Inspect artifacts locally. Do not upload proprietary samples or captures to an AI provider. Enable optional REA/MCP only after verifying the installation and real startup procedure; the current empty example configuration is not usable. Restart OpenCode after skill or configuration changes so the next session loads them.

## Pull requests

For the existing repository:

1. Keep a branch and PR focused on one contribution. Inspect the diff, stage only intended files, and exclude private research, credentials, generated output, and third-party binaries.
2. Explain the problem, scope, architectural impact, and acceptance criteria. Link a real issue if one exists.
3. List exact checks executed and results; identify skipped checks and why. Include capability-level validation and sanitized evidence for drivers.
4. Update English reference documentation, priority pt-BR guides, and the status ledger. Record incomplete translations and unresolved limitations explicitly.
5. Disclose provenance of reused material and dependency/license changes. The project license still needs maintainer confirmation.
6. Request human review. Security findings follow [SECURITY.md](SECURITY.md), not a public PR containing exploit details or sensitive evidence.

The existing GitHub issue/PR templates and CI files are placeholders, not active submission or validation infrastructure. Maintainer ownership, enforcement contacts, and CI must be established before they can be relied upon. Agents may prepare a proposal but may only open a PR or publish externally with explicit authorization.
