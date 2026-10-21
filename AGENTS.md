# Repository instructions

## Scope and truthfulness

- Read [OPENCODE_IMPLEMENTATION_MASTER.md](OPENCODE_IMPLEMENTATION_MASTER.md) in full, inspect existing files and nested instructions, and check [implementation status](docs/architecture/implementation-status.md) before work. The current execution authorizes Phase 2 (real HID discovery/transport, daemon, protected IPC, connection management, simulation), Fase 3 (storage/profiles), Fase 4 (desktop interface), the §4.4 driver system, the §4.6 domain features, the Phase 5 DRA MVP (local JSON sessions, bounded capture import, analysis, sanitized reports and a commented scaffold) and the Phase 7 AI-agent/DRA integration (typed report validation, SHA-256 provenance, non-loadable handoff and structural manifest checks) **without hardware, capture, vendor execution, packet replay or external data sharing**. Preserve SDK 1.0/Core/Registry contracts and IPC v5; do not advance to packaging/release, community/CI infrastructure or any hardware operation without new authorization.
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

## Responsible reverse engineering

DRA, REA and all reverse-engineering tools are exclusively for **peripheral protocol study, interoperability and independent driver development in authorized environments**. Read the [responsible research policy](docs/reverse-engineering/responsible-research.md) before research; all agents/skills must apply these binding rules:

1. Investigate only owned hardware or devices with adequate authorization.
2. Use legitimately sourced executables; respect copyright, licenses and applicable law.
3. Prioritize static analysis and passive USB/HID observation.
4. Never implement authentication, DRM, licensing or security-mechanism bypasses.
5. Never extract credentials, private keys or confidential manufacturer information.
6. Never develop vulnerability exploitation or unauthorized system access.
7. Never reproduce proprietary source code, including decompiled snippets, in the repository; implement drivers independently from permissible protocol facts.
8. Keep original executables and raw captures in ignored local storage; never automatically send them to models or external services. Share only authorized, reviewed, sanitized protocol findings.
9. Require explicit human authorization for active physical-device operations, with the device, action and limits identified. Ownership, discovery, a skill or an available API does not grant this authorization.
10. Suspend operations with unknown purpose or authorization and request human review before resuming. Approval of an active experiment does not permit any prohibited activity above.

## Skills

Load the matching `.agents/skills/<name>/SKILL.md` and its relevant references before the corresponding task. [Skill index](.agents/skills/README.md): `add-device` coordinates research and contribution; `reverse-engineer` analyzes authorized artifacts; `capture-protocol` collects/correlates evidence; `develop-driver` builds evidence-backed support; `validate-device` verifies support claims; `develop-feature` changes application layers; `review-contribution` reviews changes. Skills do not override user scope or authorize hardware operations. Do not invent absent DRA/REA commands, present pcap/pcapng or live capture as implemented, add MCP configuration without verified installation, or spawn subagents unless explicitly requested.

## Validation order and commands

**Current implementation:** thirteen Cargo members: SDK, Registry, Core, `peripheral-hub-transport-hid`, `peripheral-hub-ipc`, `peripheral-hub-storage`, `peripheral-hub-profiles`, `peripheral-hub-macros`, `peripheral-hub-driver-declarative` (bounded TOML manifest interpreter), the fictitious `peripheral-hub-example-native-driver` example, the `peripheral-hub-daemon` app, the standalone `device-research-assistant` and the local `peripheral-hub-agent-handoff` (`dra-agent`); plus a pnpm workspace member `@peripheral-hub/desktop` (React/TypeScript frontend) and a standalone Tauri bridge under `apps/desktop/src-tauri` that is excluded from the root Cargo workspace. See [SDK reference](docs/architecture/sdk.md), [daemon/transport guide](docs/architecture/daemon.md), [driver system](docs/architecture/driver-system.md), [desktop reference](docs/architecture/desktop.md), [DRA reference](docs/architecture/dra.md) and [agent integration](docs/architecture/agent-integration.md). The daemon has a diagnostic CLI, explicit `--simulate` mode and `serve --manifests DIR` for reviewed declarative drivers; the DRA is a separate local CLI with JSON-only capture import, deterministic analysis, sanitized reports and a commented scaffold; `dra-agent` validates a DRA report, records its SHA-256, emits a traceable non-loadable handoff and checks declarative manifests structurally. Device-manifest, profile, research-capture, research-report and research-handoff schema files are documented; Rust semantic validation is authoritative. Proprietary physical drivers, pcap/live capture, other schemas, packaging and CI remain unimplemented; the declarative interpreter supports volatile lighting only. The Tauri shell needs platform webview dependencies and is not built here. Do not claim CI/hardware compatibility passes, and never equate the frontend build/typecheck or DRA smoke test with a running desktop, driver or physical control.

Inspect package names/scripts first, then validate in this order:

1. Formatting: `cargo fmt --all --check`.
2. Rust static checks: `cargo clippy --workspace --all-targets -- -D warnings` (document justified target restrictions).
3. Compilation and tests: `cargo check --workspace`, `cargo test --workspace`; tests cover contracts, matching, inventory/logging, bounded HID I/O, local IPC, simulated lifecycle, errors, deadlines, profiles/macros, display contracts, the DRA's synthetic local pipeline and the agent handoff/skill compatibility. Hardware I/O and Windows runtime checks are opt-in, separate from software/static evidence. DRA and handoff tests prove local parsing/analysis/report/handoff behavior only; they do not validate a protocol, manifest match or device.
4. Frontend (only when implemented/in scope): `pnpm install --frozen-lockfile`, then only lint/test/build scripts actually defined in `package.json`.
5. Implemented schema/semantic validators; invalid inputs must fail. Hardware tests are explicit opt-in and never automatic CI work.

Record which checks ran, exact results, and unavailable prerequisites. No push, PR creation, releases, proprietary binary execution, or external data sharing without explicit authorization. Apache-2.0 remains proposed until confirmed.
