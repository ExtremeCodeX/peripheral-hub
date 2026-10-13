# Contributing to Peripheral Hub

[Português brasileiro](docs/pt-BR/contributors/contributing.md)

Thank you for helping build a transparent, community-maintained peripheral platform. The [master specification](OPENCODE_IMPLEMENTATION_MASTER.md) defines the product; [AGENTS.md](AGENTS.md) and the [architecture overview](docs/architecture/overview.md) define contribution boundaries.

## What contributors can do now

This stage delivers documentation and organization only. All application manifests, source placeholders, schemas, and workflows remain empty. Documentation review, translations, architectural discussion, and authorized local research are useful now. Driver/feature development below is a workflow for subsequent implementation stages, not an assertion that an SDK or tooling is available.

The proposed Apache-2.0 license is not confirmed; `LICENSE` is empty. Ask the maintainer to resolve licensing before integrating substantial code or making an official distribution. A published security contact and community conduct/enforcement policy are also pending. Be respectful, constructive, and mindful of personal information in every interaction.

## Development environment

The intended toolchain is stable Rust with Cargo, Node.js compatible with the eventual frontend dependencies, and pnpm. Windows 10/11 is the initial target; Linux/macOS portability must be preserved. Desktop system prerequisites should be taken from [Tauri 2 documentation](https://v2.tauri.app/start/prerequisites/) and recorded for the implemented target.

Versions are not pinned yet: `rust-toolchain.toml`, `package.json`, and lockfiles are empty. Do not assume a working `cargo run`, `pnpm dev`, or DRA command. When workspaces are implemented, maintainers must select versions, add only actual Cargo members, generate real lockfiles, and publish tested setup/run instructions. Commit `Cargo.lock` and `pnpm-lock.yaml` for reproducibility; do not commit dependency trees or build output.

The current checkout has no Git metadata. Once a canonical repository is established, fork it, clone your fork, and create a focused branch. Never fabricate a repository URL or maintainer identity in instructions.

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

No application checks can run against the current empty manifests. For documentation changes, inspect content, relative links, frontmatter, scope, translations, and status claims.

**Future gates, conditional on actual workspaces and tools:**

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm install --frozen-lockfile
```

Then run the lint, test, and build scripts actually defined in the frontend manifests, plus implemented schema and semantic validators. These commands are not current setup instructions. Report failures and unavailable tools honestly.

Meaningful tests should cover typed contracts, ambiguous driver matching, lengths/limits, invalid inputs, `NotSupported`, disconnects, timeouts, IPC authorization, profile migrations, and DRA correlation/sanitization. Frontend tests must distinguish simulated and physical states. Hardware tests are opt-in, outside automatic CI, with exact OS/device/firmware/capability evidence.

## Using AI skills

Read `AGENTS.md` first and load the relevant [skill](.agents/skills/README.md). The device flow is `add-device` → `reverse-engineer`/`capture-protocol` → `develop-driver` → `validate-device` → `review-contribution`; use `develop-feature` for application work. This is a coordinated procedure, not mandatory agent automation. The same evidence and review requirements apply to human-only contributions.

Inspect artifacts locally. Do not upload proprietary samples or captures to an AI provider. Enable optional REA/MCP only after verifying the installation and real startup procedure; the current empty example configuration is not usable. Restart OpenCode after skill or configuration changes so the next session loads them.

## Pull requests

When the remote repository and Git history exist:

1. Keep a branch and PR focused on one contribution. Inspect the diff, stage only intended files, and exclude private research, credentials, generated output, and third-party binaries.
2. Explain the problem, scope, architectural impact, and acceptance criteria. Link a real issue if one exists.
3. List exact checks executed and results; identify skipped checks and why. Include capability-level validation and sanitized evidence for drivers.
4. Update English reference documentation, priority pt-BR guides, and the status ledger. Record incomplete translations and unresolved limitations explicitly.
5. Disclose provenance of reused material and dependency/license changes. The project license still needs maintainer confirmation.
6. Request human review. Security findings follow [SECURITY.md](SECURITY.md), not a public PR containing exploit details or sensitive evidence.

The existing GitHub issue/PR templates and CI files are placeholders, not active submission or validation infrastructure. Maintainer ownership, enforcement contacts, and CI must be established before they can be relied upon. Agents may prepare a proposal but may only open a PR or publish externally with explicit authorization.
