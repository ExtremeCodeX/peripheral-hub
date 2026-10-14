# Repository instructions

## Scope and truthfulness

- Read [OPENCODE_IMPLEMENTATION_MASTER.md](OPENCODE_IMPLEMENTATION_MASTER.md) in full, inspect existing files and nested instructions, and check [implementation status](docs/architecture/implementation-status.md) before work. This execution authorizes Phase 2 (real HID discovery/transport, daemon, protected IPC, connection management, simulation) only. Preserve SDK 1.0/Core/Registry contracts; do not advance to Phase 3 desktop work.
- Preserve existing directories and user changes. Inspect before editing; no destructive Git cleanup, fabricated implementations, invented packets, or fabricated test results.
- Record actual changes, commands, failures, and blockers in the status ledger. Never equate directory scaffolding, simulated tests, or static analysis with working software or hardware compatibility.

## Architecture and development

- Use stable Rust/Cargo, Tauri 2 + React/TypeScript/Vite, and pnpm. No Electron, second frontend framework, or Python application runtime; Python is limited to justified laboratory tooling.
- Keep frontend → restricted Tauri API → typed, versioned IPC → user-session daemon → SDK/registry/drivers → transport boundaries. No frontend USB/HID access. DRA is a standalone development tool; REA is optional and absent from the shipped runtime.
- Use protected same-user Windows named pipes and Unix sockets; no public network listener or automatic administrator daemon. Apply least-privilege Tauri capabilities.
- Native drivers are reviewed, explicitly registered Rust crates compiled into the build, not arbitrary runtime DLLs. Declarative drivers permit only modeled operations with size, type, timeout, and rate limits; untrusted manifests cannot become arbitrary commands.
- Distinguish device model and physical instance. Match interfaces/usage and additional evidence, not VID/PID alone. Unsupported operations return `NotSupported`; transport failures must not become success.
- Use typed errors/DTOs, explicit contract/schema versions, SQLite migrations, versioned JSON profiles, and TOML typed plus semantic validation. Tokio only where useful. Add Cargo members only for implemented crates.
- Rust uses edition 2024 and stable 1.99.0 pinned in `rust-toolchain.toml`, workspace rustfmt/Clippy, documented public APIs and forbidden unsafe code. Future frontend uses strict TypeScript with configured linting. Document public contracts and architectural changes with ADRs. English is the technical reference; maintain priority public guides in pt-BR.

## Security and evidence

- Follow [SECURITY.md](SECURITY.md). Research only authorized software/hardware. Keep raw binaries, captures, personal data, credentials, and research sessions in ignored local storage; review sanitized exports before any sharing.
- Never automatically execute vendor binaries or transmit evidence to external services. Dynamic analysis, capture elevation, and physical experiments require explicit human authorization and an appropriate laboratory.
- No unknown packet replay, firmware flashing, bootloader entry, undocumented permanent-memory writes, or destructive operations. Enumeration is non-destructive. Macro MVP is editing/validation/persistence/simulation, not global input injection.
- Preserve provenance and distinguish `simulated`, `inferred`, `tested`, `validated-on-hardware`; research hypotheses use `confirmed-on-hardware`, `inferred`, `unverified`. Report firmware/platform and capability-specific evidence.

## Skills

Load the matching `.agents/skills/<name>/SKILL.md` and its relevant references before the corresponding task. [Skill index](.agents/skills/README.md): `add-device` coordinates research and contribution; `reverse-engineer` analyzes authorized artifacts; `capture-protocol` collects/correlates evidence; `develop-driver` builds evidence-backed support; `validate-device` verifies support claims; `develop-feature` changes application layers; `review-contribution` reviews changes. Skills do not override user scope or authorize hardware operations. Do not invent absent DRA/REA/scaffold commands, add MCP configuration without verified installation, or spawn subagents unless explicitly requested.

## Validation order and commands

**Current implementation:** six Cargo members: SDK, Registry, Core, `peripheral-hub-transport-hid`, `peripheral-hub-ipc`, and `peripheral-hub-daemon`. See [SDK reference](docs/architecture/sdk.md) and [daemon/transport guide](docs/architecture/daemon.md). The daemon has a diagnostic CLI and explicit `--simulate` mode; physical drivers, desktop, DRA, profiles, schemas and CI remain unimplemented. Do not run empty frontend packages or claim CI/hardware compatibility passes.

Inspect package names/scripts first, then validate in this order:

1. Formatting: `cargo fmt --all --check`.
2. Rust static checks: `cargo clippy --workspace --all-targets -- -D warnings` (document justified target restrictions).
3. Compilation and tests: `cargo check --workspace`, `cargo test --workspace`; tests cover contracts, matching, inventory/logging, bounded HID I/O, local IPC, simulated lifecycle, errors and deadlines. Hardware I/O and Windows runtime checks are opt-in, separate from software/static evidence. Profiles/DRA/frontend need later phases.
4. Frontend (only when implemented/in scope): `pnpm install --frozen-lockfile`, then only lint/test/build scripts actually defined in `package.json`.
5. Implemented schema/semantic validators; invalid inputs must fail. Hardware tests are explicit opt-in and never automatic CI work.

Record which checks ran, exact results, and unavailable prerequisites. No push, PR creation, releases, proprietary binary execution, or external data sharing without explicit authorization. Apache-2.0 remains proposed until confirmed.
