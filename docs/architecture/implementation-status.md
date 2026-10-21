# Implementation status and evidence ledger

Last updated: 2026-10-21. English is the technical reference. This ledger distinguishes delivered documentation from working software.

## Authorized scope

The current user authorized **Phase 2 — HID communication and daemon** (preserve SDK 1.0/Core/Registry; add real hidapi discovery/transport, same-user local IPC, connection management, explicit simulation and automated tests), then **Fase 3 — storage/profiles** (SQLite persistence and typed profile IPC), then **Fase 4 — desktop interface** (React/Tauri UI over the real daemon), then the **driver system** (the master's §4.4 declarative/native drivers), and then the master's **§4.6 domain features** starting with **profile application**, continuing with **generic multi-zone RGB lighting**, **macro persistence and editing** and **display contract/preview** (their "Fase 5", which is the master's out-of-scope DRA), together with a **pnpm configuration correction**. The §4.6 domain surface is now covered at the editing/persistence/contract/preview level; real device behavior remains limited to the software simulator. No commit, push or PR is authorized; DRA/research tooling, packaging/release, CI and hardware operations remain unauthorized. Phase 1–3 and documentary records below are historical evidence.

### Active Phase 2 plan

1. Audit instructions, architecture, relevant skills, contracts and current Git/tools — completed. Initial worktree clean at `6e7d642`.
2. Implement real HID discovery/transport, protected framed IPC, daemon lifecycle, explicit simulator — completed as software implementation; native report I/O and Windows runtime validation remain pending.
3. Run formatting, all-target Clippy, compilation, tests and daemon/client integration — completed. Final suite: 55 passed, zero failures/ignored; real Linux process and enumeration checks recorded below.
4. Update architecture, real commands, bilingual public status and evidence/limits — completed. 177 relative links, seven skill structures, Git whitespace and preservation/scope checks passed.

The temporary Rust 1.99.0/Zig tools remain available. Cargo tooling/cache moved to ignored `target/phase2-cargo` with the original `/tmp/opencode/cargo` path retained as a symlink after `/tmp` ran out of space. Non-destructive real HID enumeration was explicitly executed later in this session, without input/output/feature report I/O, firmware changes or private capture. No evidence was sent to external services.

### Active Phase 1 plan

1. Audit instructions, all skills, architecture/ADRs, source/manifests, Git, and tool availability — completed. Working tree initially clean at `6cfea49`; all existing Rust manifests/sources remain empty.
2. Implement only `crates/device-sdk`, `crates/device-registry`, and `crates/core` as Cargo members — completed. Synchronous typed contracts, no physical I/O or invented device protocols.
3. Test valid/invalid identities, capabilities, SDK compatibility, interface matching/ambiguity, error propagation, inventory atomicity, and structured log privacy — completed: 34 unit tests and one isolated logging integration test passed.
4. Run fmt, Clippy, `cargo check --workspace`, `cargo test --workspace`, build and API documentation — completed. Final relative-link/diff checks passed; current public guides/skills have been updated.

Rust/Cargo and a C linker were not initially on PATH. Official rustup installed stable Rust 1.99.0 with rustfmt/Clippy into temporary `/tmp/opencode/{cargo,rustup}` using `--no-modify-path`. The installer warned that `cc` was absent. Zig 0.15.2 was downloaded from the official site and extracted under `/tmp/opencode/` to provide a session-local linker. Tools are outside the repository; build/linker caches are under ignored `target/`. Actual Phase 1 results and environment adaptations appear below.

## Historical documentary baseline audit

- Read all 571 lines of `OPENCODE_IMPLEMENTATION_MASTER.md`, existing `AGENTS.md`, all seven `SKILL.md` files, architecture placeholders, manifests, representative source files, and directory inventories. There were no nested instruction files or existing ADRs in the inventory.
- Read-only recursive inventory: **55 files and 92 directories**. The master specification was the only nonempty file. All other 54 files were empty; existing organization was preserved.
- Master specification SHA-256: `ee537731a3aa19adca1bf5d81ec2fe05b9be7bc12b88a730ee0bbdb5e43ac2af`.
- Rust/Node manifests and lockfiles, Tauri configs, schema files, source placeholders, workflows, templates, CODEOWNERS, and LICENSE had no content.
- At the original audit, the checkout was not a Git repository. No canonical remote, commit baseline, or Git diff was available then. See the subsequent publication session below for the current Git state.
- Python 3.13.15 is available for local documentary checks. `rustc`, `cargo`, `node`, `pnpm`, `opencode`, and `tshark` were not found on PATH. No REA/USBPcap installation or hardware access was verified; no hardware operations were attempted.

## Historical documentary foundation delivery

| Task | State | Evidence / acceptance |
| --- | --- | --- |
| Full specification and structure audit | Completed (concluído) | Baseline inventory, original master hash, recorded tool/Git limitations |
| English and pt-BR READMEs | Completed (concluído) | Goals, roadmap, architecture, current availability, contribution and pending license |
| Agent rules | Completed (concluído) | Scope, boundaries, evidence, security, skills and conditional future validation order |
| Contribution and security policies | Completed (concluído) | English references and priority pt-BR translations; no invented private contact |
| Ignore rules | Completed (concluído) | Git matching verified for 24 excluded and 13 retained sample paths; not a comprehensive data-leak guarantee |
| Seven skill procedures and index | Completed (concluído) | Inputs/tools/steps/artifacts/stop conditions and required frontmatter checked; runtime discovery not tested without OpenCode |
| Architecture, ADR, protocol/research guides | Completed (concluído) | Required contracts/boundaries documented without implemented API/schema claims |
| Public device/user guides | Completed (concluído) | Bilingual device workflow and current availability; installation/use remain unavailable |
| Documentation consistency and preservation checks | Completed (concluído) | 123 relative links, 7 skills, Markdown whitespace, original master hash, 92-directory count, and empty non-documentary placeholders verified; manual scope/translation review |

## Recorded decisions

[ADR 0001](decisions/0001-foundation-boundaries.md) records preservation, documentation-only scope, the required stack, same-user daemon IPC, two reviewed driver paths, standalone DRA/optional REA, English/pt-BR priorities, and license uncertainty. Concrete contracts/versions are deferred until implementation, not fabricated.

No source, manifest, schema, CI, license, MCP, CODEOWNERS, or release placeholder was filled merely to look complete. Existing empty skill `references/` and `scripts/` directories remain preserved; the skills link to substantive shared guides rather than supplying dummy scripts.

## Commands and outcomes

| Command / check | Actual outcome |
| --- | --- |
| `git status --short` | Failed, exit 128: not a Git repository; discovery stopped at filesystem boundary |
| `command -v rustc cargo node pnpm python3 opencode tshark` | Only `/usr/bin/python3` returned; combined command exited 127 because a requested tool was absent |
| `python3 --version` | Passed: Python 3.13.15 |
| `python3 /tmp/opencode/check_peripheral_docs.py --baseline` | Passed: 55 files / 92 directories, only master nonempty, recorded SHA-256 and all existing directories |
| `ls /tmp/opencode` | Passed: verified the temporary parent before Git check repository creation |
| `git init --bare /tmp/opencode/peripheral-ignore-check.git` | Passed, exit 0; warned that `/app/share/git-core/templates` was absent and printed default-branch hints. Temporary repository only; project checkout remains uninitialized |
| `python3 /tmp/opencode/check_peripheral_docs.py` | Passed: 66 files / 92 directories, 123 relative links, 7 required-frontmatter/structure checks, no trailing whitespace or missing final newlines, unchanged master hash and empty non-documentary placeholders |
| `git -c core.excludesFile=/dev/null --git-dir=/tmp/opencode/peripheral-ignore-check.git --work-tree=/home/extremecodex/Documents/OpenPeripherals check-ignore --no-index --stdin` | Passed via the local check script: 24 ignored samples and 13 retained samples, including lockfiles, skills, Tauri capabilities, schemas, and textual fixtures |

Manual review checked the reference stack, boundary/error rules, evidence vocabularies, absent-runtime claims, license uncertainty, and English/pt-BR priority-guide consistency against the master document. It found and corrected one overstatement about REA being absent: its installation was not verified, rather than proven absent. Relative checks cover newly filled/created Markdown, not external URL availability or the master's conceptual example links.

### Actual file changes

**18 existing empty files filled:** root `README.md`, `README.pt-BR.md`, `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.gitignore`; all seven `.agents/skills/*/SKILL.md` files; and `docs/architecture/{overview,device-lifecycle,driver-system,ipc,security}.md`.

**11 files added:** `.agents/skills/README.md`; this status ledger; `docs/architecture/decisions/0001-foundation-boundaries.md`; `docs/protocols/specification-format.md`; `docs/reverse-engineering/research-workflow.md`; English and pt-BR `contributors/adding-a-device.md` and `users/project-status.md`; and pt-BR `contributors/{contributing,security}.md`.

The final inventory has 66 files in the same 92 directories, with 36 original placeholders still empty. No directory or original file was removed. Since all baseline files except the unchanged master were empty, inspection plus the final inventory provides a documentary change review; a real Git diff remains unavailable. Temporary verification files under `/tmp/opencode/` are outside the project and are not deliverables.

The temporary check script is local session tooling outside the repository, not shipped application code. It does not execute vendor artifacts, access hardware, fetch external data, or validate a device protocol.

Application formatting, Clippy, unit/integration/frontend tests, schema checks, builds, and launch commands were **not run**: workspaces, validators, and sources do not exist and required runtimes are missing. Hardware testing was not authorized/requested. This is not a passing software test report or CI result.

## Historical post-documentation milestones (superseded for current Phase 1)

| Priority / milestone | State | Required verifiable next step |
| --- | --- | --- |
| P0: license and governance | Blocked (bloqueado) on maintainer decisions | Confirm license; publish verified private security contact, conduct/enforcement policy, canonical repository and real owners |
| P1: real workspaces and toolchain | Blocked (bloqueado) in this scope | Authorize implementation, install/pin tools, add real Cargo/pnpm packages and actual minimal Tauri/daemon sources |
| P1: SDK, registry, transports | Blocked (bloqueado) in this scope | Implement typed contracts, safe matching, fake transport/error tests and non-destructive HID backend |
| P1: daemon, IPC, profiles | Blocked (bloqueado) in this scope | Implement same-user access checks, versioned protocol, SQLite migrations and profile integration |
| P2: desktop features | Blocked (bloqueado) in this scope | Connect to actual daemon, implement safe domain functions and accurate UX states |
| P2: DRA and validators | Blocked (bloqueado) in this scope | Implement stable research interchange, synthetic pipeline, sanitization, schema/semantic validators and real CLI commands |
| P2: community infrastructure | Partial (parcial) | Fill actual issue/PR templates and conduct/changelog policies; configure CI only for implemented workspaces; CODEOWNERS needs real identities |
| P3: hardware compatibility | Blocked (bloqueado) on evidence/authorization | Obtain target devices/firmware and approved capture/physical tests; report each capability independently |
| P3: packaging/release | Blocked (bloqueado) | Real platform builds, tested instructions, security review, confirmed license and authorized signing/publication |

The master document's phases 1–7 and minimum software checklist remain unfulfilled as software milestones. Completing parts of its documentation phase does not satisfy its workspace/build/runtime acceptance criteria.

## Translation and public guide coverage

READMEs, contribution/security priorities, device contribution, and availability guidance have pt-BR coverage. Architecture, ADRs, protocol/research procedures, and skills remain English technical references. Detailed installation, usage, profiles, compatible-device catalog, and troubleshooting manuals wait for real functionality; no empty guide is presented as complete. `CODE_OF_CONDUCT.md` and `CHANGELOG.md` remain empty files. The subsequent Git inspection found an existing Apache-2.0 `LICENSE`, preserved from the remote initial commit; explicit confirmation required by the master remains pending.

## Subsequent session: requested GitHub publication

The user explicitly authorized publication to `https://github.com/ExtremeCodeX/peripheral-hub`. The current local checkout now has branch `main`, remote `origin` pointing to that repository, and two commits: `34e3001` (remote initial commit) and `3855f4b` (`chore: initialize project structure`, containing the documentary foundation). The working tree was clean before publication preparation. Git history now permits a real diff review; previous no-Git observations above are historical evidence.

Remote `main` and HEAD were verified at `34e300146c1fbd85d4bbb43c2cbe595df9e33b73`. The remote initial commit contains a 201-line Apache-2.0 `LICENSE`. It is present locally and unchanged in `3855f4b`; it must not be replaced with the empty placeholder from the original documentary baseline. Public references were corrected for the real repository and existing license while retaining the master's explicit confirmation requirement.

### Publication checks and blocker

| Command / action | Actual outcome |
| --- | --- |
| `gh auth status` and `gh repo view ExtremeCodeX/peripheral-hub --json nameWithOwner,defaultBranchRef,isEmpty,url,viewerPermission` | Failed: `gh` was not on PATH |
| `git status --short`, `git remote -v`, `git log --oneline -10`, `git diff --stat` | Passed; clean local tree, expected origin and two-commit history |
| `git ls-remote https://github.com/ExtremeCodeX/peripheral-hub.git HEAD refs/heads/main refs/heads/master` | Passed; remote HEAD/main at the initial commit |
| `git fetch origin` | Passed; public repository readable |
| `git show --stat --oneline origin/main`, `git show origin/main:LICENSE`, `git show --stat --oneline 3855f4b`, `git diff --stat origin/main..HEAD` | Passed; preserved initial license and reviewed documentary/scaffold change inventory |
| `git diff --check origin/main..HEAD` | Passed; no diff whitespace errors |
| GitHub CLI setup | Official `cli/cli` release metadata inspected; Linux amd64 v2.102.0 archive downloaded under `/tmp/opencode/`, SHA-256 verified as `bb766f710eef8ede859c18578c72c327597cd4c8a85b06001b1f3843c6019386`, then extracted outside the project |
| `/tmp/opencode/gh_2.102.0_linux_amd64/bin/gh auth status` | Failed: no logged-in GitHub host |
| `GIT_TERMINAL_PROMPT=0 git push --dry-run origin main` | Failed: could not read Username for `https://github.com`; terminal prompts disabled. No remote update occurred |
| `python3 /tmp/opencode/check_peripheral_docs.py` (publication preparation) | Passed: 125 relative links, seven skill structures, Markdown whitespace, unchanged master, 92 non-Git directories, preserved remote LICENSE, 24 ignored and 13 retained sample paths. Git metadata excluded from documentary inventory |
| `git diff --check` and `git diff` (publication preparation) | Passed; reviewed only the ten intended documentary corrections and the publication blocker record |

**Publication is blocked (bloqueado) on GitHub authentication.** No token was requested in chat or stored in the repository, no force push was attempted, and no runtime/software functionality was added. The user must authenticate locally before an authorized push can proceed. CLI downloads/temporary tooling are not project deliverables. The missing private security channel and explicit license confirmation remain separate governance follow-ups, not claims of completed setup.

## Phase 1: actual implementation and validation

The current user's Phase 1 is narrower than the master document's phase numbering: it includes SDK/Registry/Core but excludes physical transports, daemon/IPC, frontend, persistence, DRA and distribution. No commit, push, PR, vendor execution, hardware access, capture or external evidence sharing was performed.

### Delivered functionality

| Component | State | Actual behavior and evidence |
| --- | --- | --- |
| Cargo workspace/toolchain | Completed (concluído) | Three real library members, resolver 3, edition 2024, stable Rust 1.99.0 pinned, generated lockfile, workspace Clippy/rustfmt, unsafe forbidden and public docs required |
| Device SDK 1.0 | Completed (concluído) | Separate model/unit/interface IDs; observed descriptor/usage; version envelope; typed errors/DTOs; semantic matching/capability/command/limit validation |
| Driver/transport interfaces | Completed (concluído) as contracts only | Reviewed factory/driver traits, unsupported defaults, execution provenance, HID report-ID/payload normalization and bounds wrapper; no production backend, driver or simulator |
| Device Registry | Completed (concluído) | Validated explicit registration, duplicate refusal, interface/usage/product/firmware matching, ambiguity errors, no construction during identification; factory errors propagate |
| Core inventory | Completed (concluído) | Atomic replacement of caller-supplied descriptors, unknown endpoint visibility, distinct physical units/composite interfaces, duplicate/conflicting identity refusal, snapshot change counts |
| Structured logging | Completed (concluído) | Tracing fields/events and static codes; JSON integration test verifies counts and omission of private paths/serials/unit/endpoint tokens; libraries install no global subscriber |
| Rust validation | Completed (concluído) | Final formatting, Clippy all targets, check, tests, locked build and rustdoc succeeded on Linux laboratory host |
| Documentation/skills | Completed (concluído) | SDK reference and ADR 0002 added; English/pt-BR status/setup and relevant skills updated; 146 relative links and seven skill structures checked |

Test-only IDs (`0xffff`/`0xfffe`), factories, paths, and reports are explicitly synthetic contract fixtures, never real model support or an invented proprietary packet specification. There are no public fake drivers or simulation applications.

### Validation environment and commands

`rustc --version` returned `rustc 1.99.0 (b940084d7 2026-09-28)`; `cargo --version` returned `cargo 1.99.0 (5f94df478 2026-08-27)`. Zig reported `0.15.2`. The temporary wrapper `/tmp/opencode/phase1-cc` executes the downloaded official `zig cc` solely as a linker. Zig is not a project dependency.

Before validation, the following environment was exported in each terminal invocation:

```sh
export PATH="/tmp/opencode/cargo/bin:$PATH"
export CARGO_HOME=/tmp/opencode/cargo
export RUSTUP_HOME=/tmp/opencode/rustup
export RUSTUP_TOOLCHAIN=stable
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/opencode/phase1-cc
export ZIG_GLOBAL_CACHE_DIR=/home/extremecodex/Documents/OpenPeripherals/target/zig-cache
```

The temporary `stable` toolchain resolves to the same verified 1.99.0 release as the repository pin. This explicit override avoided a second toolchain download after the limited `/tmp` tmpfs ran out of space. No rustup profile or project-local linker configuration was changed to depend on `/tmp`. Normal contributors use the pinned toolchain and their native linker as documented in [SDK setup](sdk.md).

| Command / check | Final outcome |
| --- | --- |
| `cargo fmt --all` | Passed; formatted all implemented crates |
| `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no target restrictions or warning suppressions |
| `cargo check --workspace` | Passed, exit 0; all three libraries |
| `cargo test --workspace` | Passed, exit 0; SDK 18 unit tests, Registry 8, Core 8, logging integration 1: **35 passed / 0 failed / 0 ignored**. Three doc-test suites have zero tests; not counted as extra test evidence |
| `cargo build --workspace --locked` | Passed, exit 0; library artifacts only, not an installable application |
| `cargo doc --workspace --no-deps` | Passed, exit 0; generated all three public API references under ignored `target/doc/` |
| `python3 /tmp/opencode/check_phase1_docs.py` | Passed: 146 relative links, seven skill structures, three real members, unchanged master/LICENSE, untouched protected application/schema/CI placeholders and ignored build/cache paths |
| `git diff --check`, `git diff --stat`, `git status --short` | Passed; no tracked whitespace errors, only intended Phase 1 code/config/docs changed, new crates/SDK reference/ADR visible as untracked additions. No files were staged or committed |

### Failures encountered and corrected

1. `git status --short` was clean; `git log --oneline -5` established the current baseline `6cfea49`. Initial `command -v rustc cargo rustup node pnpm python3` found only Python; the C toolchain lookup also returned no compiler/linker. Tools were supplied outside the repository.
2. Running the installed rustc against the existing empty `rust-toolchain.toml` failed with “empty toolchain override file detected”. Filling the real 1.99.0 toolchain file fixed the invalid configuration.
3. Initial pinned `cargo fmt --all` triggered a duplicate release installation and failed with “No space left on device” while downloading rustfmt. `df -h` identified the 1.4 GiB `/tmp` tmpfs, while the workspace filesystem had 87 GiB available. Validation used the already installed stable 1.99.0 with an explicit environment override, and put Zig cache under ignored workspace `target/`.
4. First Clippy/compilation run caught an extreme-comparison lint for the 1.0 minor-version gate and a borrow-check failure in a command-boundary test. Version comparison and the test borrow scope were corrected; the next run passed all 34 then-existing tests.
5. Review removed unused tracing attribute/ANSI default features and corrected the Core's physical grouping so HID/non-HID interfaces can share a unit; a new unit test covers that case. A subsequent parallel unit run failed to capture the expected structured inventory event. Logging assertions were moved to a separate integration-test binary with an isolated subscriber instead of relying on parallel unit-test callsite state. The final complete suite then passed 35 tests without serializing the overall test runner or weakening event/privacy assertions. Final contract review also applied unknown-field rejection to error DTOs and extended the existing serialization test accordingly.

### Important limits and next review

- No enumeration/`hidapi` adapter, physical opening/closing, connection detection, daemon, IPC or runtime simulator was implemented. Inventory diffs are differences in supplied snapshots, not OS hotplug.
- Capabilities and a typed lighting request are contracts/validators, not RGB/display/macro/profile functionality. SQLite, manifest parsing/interpreting and JSON Schemas remain future work.
- Serde parsing is not a bounded external wire parser or sanitized export. Future IPC/import layers must cap input before allocation; reviewed drivers/backends must enforce real deadlines, rates and packet safety.
- Windows/macOS toolchains, installers, frontend builds, CI and physical devices were not tested. Linux library checks do not establish cross-platform application or hardware support.
- `LICENSE` and all pre-existing placeholders outside Phase 1 are preserved. Explicit license confirmation, private security channel, conduct policy, and real ownership remain governance follow-ups.

**Next action:** user review of Phase 1. Only after approval may Phase 2 add non-destructive HID discovery/backend, connection management, protected same-user IPC, daemon and explicit simulators. Later frontend/driver/domain/DRA/CI phases follow their own reviewed scope.

### Relevant changed files

Workspace/toolchain: `Cargo.toml`, generated `Cargo.lock`, `rust-toolchain.toml`. Implemented packages: `crates/device-sdk/{Cargo.toml,src/}`, `crates/device-registry/{Cargo.toml,src/}`, `crates/core/{Cargo.toml,src/,tests/structured_logging.rs}`. Architectural reference: `sdk.md`, `decisions/0002-sdk-and-registry-contracts.md`, updated overview/lifecycle/driver-system and this ledger. Current status/setup: root READMEs, CONTRIBUTING/AGENTS, priority English/pt-BR contributor/user guides, skill index and add-device/develop-driver/develop-feature prerequisites. No application/schema/CI placeholders were filled.

## Phase 2: HID communication and user-session daemon

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| Preserved Phase 1 contracts | Completed (concluído) | SDK 1.0, Core and Registry source/manifests unchanged; all 35 original tests still pass |
| HID discovery | Completed for current Linux enumeration | Real hidapi initialization/refresh; eight interfaces observed; no model selected and no physical control capability claimed |
| HID report transport | Implemented; physical I/O validation partial (parcial) | Actual hidapi open/read/write/feature delegation, reviewed policy, report-ID normalization, rate/size limits, bounded worker and handle retirement. Tests use injected backend behavior; no physical report operation ran |
| Local IPC | Implemented; Windows runtime partial (parcial) | Version 1, bounded frames/DTOs, deadlines/correlation, same-user Unix credentials/modes/lock and safe Windows SID/DACL/process checks. Unix sockets executed; Windows all-target check/Clippy only |
| Daemon/lifecycle | Completed as Linux software | Serialized bounded manager actor, polling, explicit matched opening, invalidation/closing on removal/change/fault, journal, reconnect and signal shutdown |
| Simulator | Completed (concluído), simulated evidence only | Explicit `--simulate`, fictitious registered model, bounded memory transport, validated lighting/state, controlled presence and failure injection; never a physical fallback |
| Diagnostics | Completed on Linux | Binary `--help`, physical `enumerate`, `serve`, typed health/list/get/capabilities/state/lighting/connect/disconnect/events and simulation presence commands |
| Documentation and relevant skills | Completed (concluído) | Real setup/run commands/limits, IPC reference, ADR 0003, pt-BR daemon guide; 177 links and seven skill structures checked |

Physical mode has an empty Registry and exposes unknown/discovered endpoints only. Real opening/control is not claimed merely because enumeration or synthetic tests pass. The named-pipe backend has not run on Windows and is not certified as runtime-compatible. No frontend, SQLite, profile storage, declarative interpreter, proprietary driver, DRA, CI or release was added.

### Commands and actual outcomes

Validation uses the same explicit environment as Phase 1: temporary Rust stable resolves to 1.99.0, host linker `/tmp/opencode/phase1-cc` delegates to Zig 0.15.2, and caches/build output remain ignored. The Cargo path is now a symlink into `target/phase2-cargo`; this is session tooling, not a project dependency or setup requirement.

| Command / action | Outcome |
| --- | --- |
| `git status --short`, `git log --oneline -3` | Passed; initial clean worktree at `6e7d642` (`feat: implement core and device sdk foundation (#1)`) |
| Tool lookup and `rustc --version` | No Rust/C compiler on original PATH; temporary rustc confirmed 1.99.0 |
| `cargo fetch` | First dependency fetch succeeded; later all-platform fetch failed unpacking Windows i686 import libraries with “No space left on device” in `/tmp` |
| `ls target`; `mv /tmp/opencode/cargo target/phase2-cargo && ln -s /home/extremecodex/Documents/OpenPeripherals/target/phase2-cargo /tmp/opencode/cargo` | Passed; only temporary session tooling/cache relocated to ignored output; no source cleanup or user implementation overwritten |
| `rustup target add --toolchain stable x86_64-pc-windows-gnu` with temporary homes | Passed; installed standard library for static cross-target checks |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0, after final code changes |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no warning suppressions or host target restrictions |
| `cargo check --workspace` | Passed, exit 0; six implemented members |
| `cargo test --workspace` | Passed, exit 0: **55 passed / 0 failed / 0 ignored**. SDK 18, Registry 8, Core/logging 9, HID 5, IPC 6, daemon 9. Empty doc-test suites are not counted as extra tests |
| `cargo build --workspace --locked` | Passed, exit 0; actual host daemon binary built |
| `cargo doc --workspace --no-deps` | Passed, exit 0; six API references generated in ignored `target/doc/` |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-gnu` | Passed, exit 0; static compilation/type checking only, not linked Windows execution or runtime tests |
| `cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu -- -D warnings` | Passed, exit 0; static Windows lint evidence only |
| `python3 /tmp/opencode/phase2_cli_smoke.py` | Passed using separate native daemon/client processes in explicit simulation: help/health/list/duplicate refusal/connect/capabilities/lighting/state/removal/reappearance/events/SIGTERM/socket cleanup |
| `target/debug/peripheral-hub-daemon enumerate` | Passed, exit 0; observed eight real HID interfaces, all unknown/discovered in physical mode, no raw locator/serial exported |
| `python3 /tmp/opencode/phase2_physical_inventory_smoke.py` | Passed on final binary: physical enumerate and daemon IPC each showed eight unknown/discovered interfaces; connect on an unknown endpoint and simulation in physical mode returned NotSupported; SIGINT/socket cleanup passed. No input/output/feature report operation or physical driver validation |
| `python3 /tmp/opencode/check_phase2_docs.py` | Passed: 177 relative links, seven skill structures, six real members, unchanged SDK/Core/Registry/master/LICENSE/toolchain and preserved out-of-scope placeholders; temporary tooling/build paths remain ignored |
| `git diff --check`, `git diff --stat`, `git status --short` | Passed; only intended workspace/HID/IPC/daemon and documentation additions/updates visible. No files staged, committed or pushed |

The two Python scripts are ephemeral laboratory verification under `/tmp/opencode`, not a Python application runtime or shipped tools. Real discovery/process checks were explicitly invoked outside Cargo tests. The automatic suite uses simulation/test doubles and local IPC only; it never enumerates/opens physical devices.

### Failures corrected and design review

- Initial all-target Clippy reported collapsible conditionals in socket cleanup and daemon lifecycle; they were rewritten with no warning suppression.
- Initial local integration tests failed because Rust tempfile directories inherited permissive permissions in this environment. Tests now explicitly provision private mode-0700 endpoint directories; production authorization was retained, not weakened. Subsequent complete suites passed.
- A docs.rs guessed pipe-options URL returned 404. Actual downloaded interprocess/hidapi/sysinfo sources were inspected and Windows code checked instead of inventing API signatures. Windows identity APIs use safe dependency wrappers; project unsafe remains forbidden.
- Physical device parent grouping was deliberately kept conservative: different paths are not merged by VID/PID/serial. Duplicate usage collections collapse to unknown usage rather than guessed protocol matches. Report I/O policies/rates are explicit and no physical policy is guessed.
- Deadline review added exact remaining-wait checks/handle retirement and tests of expired queued mutation cancellation. A fresh random source epoch prevents reused endpoint IDs after restart; `Health.session_id` instructs event clients to reset cursors. Restart integration tests reject stale IDs.
- Removed an unnecessary fingerprint/hash dependency; OS entropy is used only for ephemeral identity namespaces. OS backend error messages are sanitized before logs/IPC and private fields are excluded from device summaries.

### Limitations and pending priorities

1. **Physical report I/O and reviewed drivers:** input/output/feature behavior, report lengths, Windows-native count/padding conventions, firmware and capability-specific operations require approved hardware/protocol evidence. This execution only tested non-destructive enumeration physically; no compatibility entry is created.
2. **Windows runtime:** verify actual same/different-user SID/DACL access, named-pipe reconnect/duplicate/shutdown, process-owner lookup and device permissions on Windows 10/11. Cross-target check/Clippy are not runtime evidence. Linux peer credentials were exercised with same-user processes; no privilege change/cross-user physical test was attempted.
3. **Discovery/lifecycle constraints:** composite/dongle grouping across paths is incomplete; interface count is not physical unit count. Polling can miss short transitions. Known records are retained on enumeration failure, but list/open/driver operations surface the error rather than claiming an empty success.
4. **Native cancellation:** kernel HID writes/features can outlive deadlines; callers receive Timeout and handles retire, but cancellation cannot be guaranteed. An already executing mutation has an unknown outcome and must not be replayed. A stalled OS operation can retain its worker until the OS returns/process exits. No physical mutation was executed here.
5. **Later scope/governance:** desktop Phase 3, profiles/SQLite, DRA, manifests/interpreter, CI/packaging, private reporting channel and explicit license confirmation remain pending. Do not advance before user review.

### Relevant files and decisions

Added real members `crates/transport-hid/{Cargo.toml,src/}`, `crates/ipc/{Cargo.toml,src/,tests/}`, and `apps/daemon/{Cargo.toml,src/,tests/}`; updated root workspace/generated lockfile. Preserved SDK/Core/Registry, toolchain, master and license. Added [daemon guide](daemon.md), [IPC reference](ipc.md), [ADR 0003](decisions/0003-hid-daemon-local-ipc.md) and [pt-BR usage guide](../pt-BR/users/daemon.md), plus current public status/security/contribution/skill prerequisite updates. No commit, push, PR, vendor binary execution, raw capture, firmware action or external evidence upload occurred.

## Responsible reverse engineering policy (documentation-only)

| Task | State | Evidence / acceptance |
| --- | --- | --- |
| Binding RE policy | Completed (concluído) as documentation | Added [responsible-research.md](../reverse-engineering/responsible-research.md) (English reference) with purpose, ten rules, applied workflow and evidence handling; pt-BR summary in [pt-BR contributors guide](../pt-BR/contributors/responsible-research.md) |
| Agent rules | Completed (concluído) as documentation | `AGENTS.md` gained a "Responsible reverse engineering" section listing the ten binding rules and linking the policy |
| Security alignment | Completed (concluído) as documentation | `SECURITY.md`, `docs/architecture/security.md`, and `docs/pt-BR/contributors/security.md` cross-reference the policy (static/passive first, no bypass/secrets/exploitation/unauthorized access/proprietary source, no automatic external upload, human review) |
| Workflow and skills | Completed (concluído) as documentation | Research workflow boundary plus each skill's Tools/Stop conditions updated; `.agents/skills/README.md` new mandatory-policy section |

This entry records a documentation-only change. No Rust source, manifest, IPC/daemon behavior, SDK/Core/Registry contract, master specification or license was modified, and no software, hardware or CI validation is claimed. Relative-link and scope checks are recorded below. The earlier ephemeral `check_phase2_docs.py` script no longer exists in this session; the check below was performed with a fresh local-only link checker.

| Command / check | Outcome |
| --- | --- |
| `python3 /tmp/opencode/check_relative_links.py` | Passed: 110 relative links across the 16 policy/agent/security/skill files resolve; no broken references |
| `git status --short`, `git diff --check` | Passed; only intended Markdown additions/updates (no `crates/`, `apps/`, `Cargo.*` or schema changes); no tracked whitespace errors and no files staged or committed |

## Storage and profiles (master Fase 3: "Daemon, IPC e perfis")

The current user authorized completing the master document's Fase 3, which adds real persistence: `crates/storage` (SQLite connection and forward-only migrations) and `crates/profiles` (schema-versioned profile model and repository), plus profile operations in IPC/daemon and automated tests. Phase 1 and Phase 2 records above are preserved historical evidence. No commit, push, PR, vendor execution, hardware access or external data sharing occurred.

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| Storage library | Completed (concluído) | `peripheral-hub-storage`: single owned `rusqlite` 0.40 connection, `foreign_keys` ON, 5-second busy timeout, `PRAGMA user_version` migrations that are strictly increasing, run per-migration in a transaction, and reject a newer-than-supported schema without downgrade. 5 unit tests |
| Profile library | Completed (concluído) | `peripheral-hub-profiles`: `ProfileId` token validation, `Profile`/`ProfileSummary`/`ProfileFile`, `PROFILE_SCHEMA_VERSION = 1`, JSON import/export with unknown-field/version rejection, and a `ProfileRepository` (save upsert/get/list/delete/count) that re-validates on read. 7 unit tests |
| IPC contract | Completed (concluído) | `IPC_VERSION` raised to 2; typed `ListProfiles`/`GetProfile`/`SaveProfile`/`DeleteProfile`, `Reply::Profiles`/`Reply::Profile`, and `Fault::Profile(ProfileFault)` with `Invalid`/`UnsupportedVersion`/`NotFound`/`Storage`. No raw packets or file paths in the wire contract |
| Daemon integration | Completed (concluído) | Manager holds a `ProfileRepository` (`with_profiles`), handles the profile operations, and never converts a fault into success. `serve` opens the per-user SQLite file (default path or `--profiles PATH`); CLI adds `profiles`/`profile-get`/`profile-save`/`profile-delete` |
| Tests | Completed (concluído) | Suite grew 55 → 69: storage 5, profiles 7, daemon `manager` +1, IPC roundtrip +1. Integration tests exercise profile save/list/get/delete and invalid/not-found paths over local IPC |
| Documentation | Completed (concluído) | [ADR 0004](decisions/0004-storage-and-profiles.md); updated [daemon](daemon.md), [IPC](ipc.md), [overview](overview.md), [SDK](sdk.md), [security](security.md), READMEs, project-status pages, pt-BR daemon guide and skills index |

Packages added: `crates/storage`, `crates/profiles`. The workspace now has eight implemented members (SDK, Registry, Core, HID transport, IPC, storage, profiles, daemon app). SDK 1.0/Core/Registry source contracts and the empty physical registry are unchanged.

### Commands and actual outcomes

Validation reused the temporary Rust 1.99.0/Zig tools in ignored `target/` (`RUSTUP_HOME=target/phase3-rustup`, `CARGO_HOME=target/phase2-cargo`, linker shim `target/phase3-cc`). The shim was extended to translate `x86_64-pc-windows-gnu` for Zig, and the Windows standard library was added with `rustup target add x86_64-pc-windows-gnu`.

| Command / action | Outcome |
| --- | --- |
| `rustc --version`, `cargo --version` | `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)` |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0, after final changes |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no target restrictions or warning suppressions |
| `cargo check --workspace` | Passed, exit 0; eight members |
| `cargo test --workspace` | Passed: **69 passed / 0 failed / 0 ignored**. SDK 18, Registry 8, Core+logging 9, HID 5, IPC 6, storage 5, profiles 7, daemon 11 |
| `cargo build --workspace --locked` | Passed, exit 0; host daemon binary built |
| `cargo doc --workspace --no-deps` | Passed, exit 0; eight API references generated under ignored `target/doc/` |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-gnu` | Passed, exit 0; static cross-target type checking only, including the new C-backed `rusqlite`/`libsqlite3-sys` |
| `cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu -- -D warnings` | Passed, exit 0; static Windows lint evidence only |
| `/tmp/opencode/phase3_profiles_smoke.sh` | Passed with separate native daemon/client processes in explicit simulation: `profile-save`/`profiles`/`profile-get`/`profile-delete`; a 65-byte name returned `Fault::Profile("invalid")`; profiles persisted across a daemon restart; deleting removed the row |
| `python3 /tmp/opencode/check_all_links.py` | Passed: 235 relative links across all 42 repository Markdown files resolve (whole-repository scan superseding the earlier 16-file policy-only check) |
| `git diff --check`, `git status --short` | Passed; no tracked whitespace errors, intended source/config/docs changes only, no files staged, committed or pushed |

### Failures corrected and design review

- Initial `cargo check` failed because `ProfileFault` lacked a `Display` implementation for the `thiserror` format argument. A stable `Display` returning fixed codes was added rather than dropping the detail.
- `prepare_cached` was unavailable because the `rusqlite` `cache` feature is not enabled; repositories use `prepare` only.
- `impl From<rusqlite::Error> for ProfileError` and a `tempfile` dev-dependency were added after the first compile of the profiles tests failed.
- The existing daemon `manager` test assumed `GetProfile`/`SaveProfile` returned `NotSupported`; it was rewritten to assert real persistence, contract rejection and `NotFound` behavior.
- The session-local compiler shim originally only translated `-unknown-`; the Windows target passed `x86_64-pc-windows-gnu`, so the shim gained a `-pc-windows-` → `-windows-` translation. This is ephemeral tooling outside the repository.

### Limitations and pending priorities

1. **Single connection:** storage keeps one connection and does not support concurrent multi-process writers or pooling; all persistence runs on the serialized daemon manager worker.
2. **Forward-only migrations:** only `PRAGMA user_version` and forward migrations exist; there is no down-migration, schema diffing or automatic backup. A newer on-disk schema is refused, not migrated.
3. **No profile import/export CLI or desktop binding:** `Profile::to_json`/`from_json` exist in the library, but there is no user-facing import/export command or UI, and no model-driven application of a saved profile to a connected device.
4. **No encryption or sanitization of stored profiles:** the SQLite file relies on OS file permissions on the same-user data path; it is not encrypted.
5. **Windows evidence is static:** the Windows target check/Clippy including the C-built SQLite are compilation evidence only, not Windows runtime or DACL validation.
6. **Later scope/governance:** desktop UI, DRA, declarative interpreter, manifest/interchange schemas, CI/packaging, private reporting channel and explicit license confirmation remain pending.

### Relevant files and decisions

Added `crates/storage/{Cargo.toml,src/}`, `crates/profiles/{Cargo.toml,src/}`; updated root `Cargo.toml`/`Cargo.lock`, `crates/ipc/{Cargo.toml,src/protocol.rs}`, `apps/daemon/{Cargo.toml,src/main.rs,src/manager.rs,tests/manager.rs,tests/ipc_roundtrip.rs}`. Documentation: [ADR 0004](decisions/0004-storage-and-profiles.md), `daemon.md`, `ipc.md`, `overview.md`, `sdk.md`, `security.md`, READMEs, project-status pages, pt-BR daemon/contributing guides and skills index. SDK/Core/Registry, toolchain, master and license are unchanged. No commit, push, PR, vendor binary execution, raw capture, firmware action or external evidence upload occurred.

## Desktop interface (master Fase 4: "Interface de usuário")

The current user authorized the master document's Fase 4: a React/Tauri 2 desktop that operates against the real daemon with the simulated driver, including profiles. Phase 1–3 records above are preserved historical evidence. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| pnpm workspace | Completed (concluído) | Root `package.json`, `pnpm-workspace.yaml` and `@peripheral-hub/desktop`; `pnpm-lock.yaml` regenerated (the tracked file was an empty placeholder); esbuild build scripts approved via `allowBuilds` (the current pnpm syntax; maps to `onlyBuiltDependencies`) |
| Frontend types/services | Completed (concluído) | `src/types/ipc.ts` mirrors IPC v2; `DaemonTransport`/`TauriTransport`, `IpcClient` (version check, `Ok`/`Err` unwrap, discriminant assertions), typed errors, view models and `DaemonController` |
| State and screens | Completed (concluído) | Observable store + `useSyncExternalStore` hook with injectable transport; devices, details, lighting, profiles, macros, displays and diagnostics screens with empty/loading/error/unsupported states |
| Profile and macro logic | Completed (concluído) | Versioned profile JSON import/export and macro validate/serialize/simulate in pure TypeScript; macros never inject input and displays never transmit frames |
| Restricted Tauri bridge | Source complete (código); not built (não compilado) | `apps/desktop/src-tauri` single `daemon_call(Operation) -> Response` command using `Endpoint::default_for_user()` and a 5 s deadline; standalone manifest excluded from the root workspace |
| Frontend validation | Completed (concluído) | `tsc --noEmit` clean; Vitest **53 tests across 9 files** (including `DaemonController` and `tauriTransport` flow tests and 13 screen/render tests); production `vite build` succeeded |
| Host runtime round-trip | Tested (host, user-run) | Rebuilt host shell launched against a live `serve --simulate` daemon; the Devices screen listed the simulated interface (`simulated-…-interface-1`, `ffff:fffe`, driver `simulation:peripheral`), a **simulated lighting** change (RGB/brightness) was applied, and a saved **profile persisted across a daemon restart**, all through `daemon_call` → local IPC → daemon → `simulation:peripheral`. Confirms the end-to-end desktop→daemon path incl. profile persistence in simulation; macros/displays remain editing/simulation-only and unexercised on the host |
| Tauri manifest graph | Resolved (resolvido) | `cargo metadata` on `apps/desktop/src-tauri` succeeds (standalone workspace member, `tauri 2.12.4`, `tauri-build 2.7.3`, path `peripheral-hub-ipc` with workspace-inherited fields); compilation still blocked by absent webview packages |
| Rust workspace | Completed (concluído) | `fmt`/`clippy -D warnings`/`check`/`test` still pass after adding the workspace `exclude`; suite unchanged at 69 tests |
| Documentation | Completed (concluído) | [ADR 0005](decisions/0005-desktop-ui.md), [desktop reference](desktop.md), pt-BR desktop guide; updated overview/IPC/daemon/READMEs/project-status/AGENTS/skill |

### Commands and actual outcomes

Session Node/pnpm were installed under ignored `target/phase4-node` (Node v24.22.0, npm 11.19.1, pnpm 10.34.7). Rust validation reused the Phase 3 toolchain (`RUSTUP_HOME=target/phase3-rustup`, `CARGO_HOME=target/phase2-cargo`, linker shim `target/phase3-cc`).

| Command / action | Outcome |
| --- | --- |
| `pnpm install` | First run ignored the empty tracked lockfile; after approving the esbuild build script (originally written as `pnpm.onlyBuiltDependencies`, later folded into the current `allowBuilds` syntax), esbuild 0.21.5/0.25.12 postinstall succeeded and the lockfile was written |
| Align Vite/Vitest | Initial `vite ^6` conflicted with `vitest 2.1.8` (duplicate Vite 5/6 types broke `tsc`); pinned `vite ^5.4.11`, after which typecheck passed |
| `pnpm --filter @peripheral-hub/desktop typecheck` | Passed, exit 0 (`tsc --noEmit`) |
| `pnpm --filter @peripheral-hub/desktop test` | Passed: **50 tests / 8 files** (viewModel 6, profileFile 5, macroModel 6, daemonStore 4, ipcClient 6, daemonController 7, screens 13, Badge 3) |
| `cargo metadata --manifest-path apps/desktop/src-tauri/Cargo.toml` | Passed after fixing a wrong relative path; resolved 429 packages including `peripheral-hub-ipc`, `tauri 2.12.4` and `tauri-build 2.7.3`. Resolution/type-graph only, not compilation |
| Host `cargo run --manifest-path apps/desktop/src-tauri/Cargo.toml` (user-run, webview deps installed) | Compiled all Linux/webkit/GTK dependencies; failed only at `tauri::generate_context!` because no app icon was configured (`bundle.icon` empty → missing `icons/icon.png`). Icons and `bundle.icon` were then added; re-run pending |
| `pnpm --filter @peripheral-hub/desktop build` | Passed; `dist/index.html`, `index-*.css` (3.93 kB) and `index-*.js` (177.23 kB) emitted |
| `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; eight members, unchanged |
| `cargo check --workspace` | Passed after adding `exclude = ["apps/desktop/src-tauri"]` |
| `cargo test --workspace` | Passed: **69 passed / 0 failed / 0 ignored**, unchanged |
| `python3 /tmp/opencode/check_all_links.py` | Passed: 270 relative links across 45 repository Markdown files resolve |

The sandbox here cannot build the shell (`pkg-config`, `gtk+-3.0` and `webkit2gtk-4.1` are absent). A **user-run host** build (webview dependencies installed) compiled and launched the shell, and against a live `serve --simulate` daemon the device list/details rendered and a simulated lighting operation completed. Profile persistence and macro/display checks on the host are still pending; Windows remains static-evidence only.

The daemon's per-second `inventory_replaced` line was confirmed to be the normal 1 s enumeration heartbeat, not a fault. To keep terminals usable it now logs at `INFO` only when the inventory actually changes (added/removed/updated non-zero) and at `debug` otherwise; `cargo fmt`, `clippy -D warnings` and the workspace tests (including `structured_logging.rs`, which exercises the change case) still pass.

### Follow-up validation session

A later request to "conclude Phase 3 by implementing the desktop app" was reconciled against the master: in this repository the desktop is **Fase 4** and Fase 3 is daemon/IPC/profiles, and the desktop under `apps/desktop/` is already implemented and committed (`c93025b` on `feat/desktop-complete`, tree clean). No desktop implementation was missing, so this session only re-ran validation and tightened documentation rather than rewriting existing work.

| Command / action | Outcome |
| --- | --- |
| `pnpm install --frozen-lockfile` (pnpm 10.34.7) | Passed; lockfile up to date, esbuild postinstall ran. Needed `CI=true` only because a non-TTY purge of the existing `node_modules` was pending |
| pnpm config audit | `pnpm-workspace.yaml` `allowBuilds: {esbuild: true}` is the current pnpm syntax (maps to `onlyBuiltDependencies`); `package.json` `pnpm.onlyBuiltDependencies` is the older equivalent. No obsolete keys. Correction (2026-10-16): this row originally stated that `minimumReleaseAge` was not configured; it is now configured as `minimumReleaseAge: 1440` in `pnpm-workspace.yaml`, so that earlier statement is stale |
| `pnpm build` / `pnpm test` | Added root convenience scripts (`build`/`test`/`typecheck`) since `pnpm build` previously failed with no root script. Build passed; **53 tests / 9 files** pass |
| `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0 |
| `cargo check --workspace --locked` | Passed, exit 0 |
| `cargo test --workspace --locked` | Passed: **69 passed / 0 failed / 0 ignored** |
| `cargo build --workspace --locked` | Passed, exit 0 |
| Native-prerequisite probe (sandbox) | `pkg-config`/`pkgconf`, `cc`/`gcc` and any distro package manager are absent; `gtk+-3.0`, `webkit2gtk-4.1`, `javascriptcoregtk-4.1`, `libsoup-3.0` all missing. Native Tauri build is impossible in this sandbox and is documented as such |

The required Arch Linux host packages are documented in [desktop.md](desktop.md) (`base-devel`, `pkgconf`, `gtk3`, `webkit2gtk-4.1`).

### Failures corrected and design review

- The tracked `pnpm-lock.yaml` was an empty placeholder that pnpm reported as broken; it was regenerated rather than fabricated.
- The initial Vite 6 / Vitest 2 version mismatch produced a large plugin type error; the frontend pinned Vite 5 to match Vitest instead of suppressing the error.
- One `daemonStore` test asserted the wrong post-unsubscribe state; the assertion was corrected (the unsubscribe check itself passed).
- `cargo metadata` on the Tauri shell failed because its `peripheral-hub-ipc` path dependency was written as `../../crates/ipc` (resolving to `apps/crates/ipc`); it was corrected to `../../../crates/ipc` and the manifest graph then resolved. This was a real build-blocking defect caught before any webview-host build.
- The screen render tests required `useSyncExternalStore` to have a server snapshot; the provider gained an optional `initialState` and a third `getServerSnapshot` argument, which also removes a latent server-render crash.
- A host build (user-run, webview dependencies installed) compiled every shell dependency and then panicked in `tauri::generate_context!`: no application icon was configured, so it tried to open the missing `icons/icon.png`. The `bundle.icon` list was set and a small generated PNG icon set (`icons/{32x32,128x128,128x128@2x,icon}.png`) was added.
- After the icon fix the user-run host shell launched but the webview showed "Could not connect to localhost: Connection refused": a **debug** build loads `build.devUrl` (the Vite dev server) instead of the embedded `frontendDist`, and the host has no Vite/pnpm running. Root cause: Tauri selects the dev URL from the `tauri` crate's `custom-protocol` feature (`dev = !custom-protocol`), not from the Cargo profile; `tauri = { features = [] }` leaves `cfg(dev)` always on, so even `--release` loaded `devUrl`. `devUrl`/`beforeDevCommand` were removed from `tauri.conf.json` so the embedded `frontendDist` is used regardless of the dev flag; the desktop guides document this, the `touch src/main.rs` recompile caveat, and how to re-add `devUrl` for live-reload. Runtime GUI evidence against the daemon is still pending a launch of the rebuilt shell.
- On the first successful host launch the UI rendered but always showed "Daemon unavailable" and a generic "Unexpected error." on Devices, with Retry not helping. Cause: the Tauri `invoke` promise rejects with the command's Rust error value (a plain string), while `errorMessage` only recognized `Error`/`BridgeError`/`DaemonFaultError`, so the bridge's real message was discarded. `tauriTransport` now wraps any rejection in a `BridgeError`, preserving the bridge text (for example "No local daemon is reachable…"), and a `tauriTransport.test.ts` covers forwarding, message preservation and the non-string fallback (53 frontend tests pass).
- A `ProfilesScreen` draft used a non-narrowing optional access for `device_model`; it was rewritten with a local constant, and `DaemonController.getProfile` was added for screen use.
- The Tauri shell was deliberately excluded from the root workspace rather than forcing webview dependencies into `cargo check --workspace`.

### Limitations and pending priorities

1. **Desktop runtime verified in simulation (host):** the sandbox here cannot compile the shell (no webview), but a user-run host build launches, renders the UI, lists the simulated device, applies a simulated lighting change and persists a profile across a daemon restart. The daemon and desktop must be started manually in separate terminals (no installer/single-instance/auto-start); if the daemon stops the UI shows "Daemon unavailable". No physical device was touched and no installer exists.
2. **GUI evidence is simulation-only:** typecheck, 53 Vitest tests and a production build pass, and the host run above is recorded; macros and displays were not exercised on the host and remain editing/validation/persistence/simulation with no hardware action. Component tests use `react-dom/server` because no browser/jsdom environment is configured.
3. **No schema/interchange work:** profile and macro JSON are client-side versioned documents; none are shipped daemon commands, and no model-driven application of a profile to a device exists.
4. **Macros/displays are not hardware actions:** macros are editing/validation/persistence/simulation only, and displays transmit no frames.
5. **Later scope/governance:** DRA, declarative interpreter, manifest/interchange schemas, CI/packaging, signed releases, private reporting channel and explicit license confirmation remain pending.

### Relevant files and decisions

Added `package.json` (root, with pnpm build approvals), `pnpm-workspace.yaml`, regenerated `pnpm-lock.yaml`, and `apps/desktop/{package.json,tsconfig.json,vite.config.ts,index.html,src/}` (types, services, stores, hooks, components, features and Vitest tests, including `src/services/daemonController.test.ts` and `src/features/screens.test.tsx`) plus `apps/desktop/src-tauri/{Cargo.toml,build.rs,tauri.conf.json,capabilities/default.json,src/main.rs}`. Updated root `Cargo.toml` (`exclude`). Documentation: [ADR 0005](decisions/0005-desktop-ui.md), `desktop.md`, pt-BR `users/desktop.md`, `overview.md`, `ipc.md`, `daemon.md`, READMEs, project-status pages, `AGENTS.md`, `CONTRIBUTING.md`, pt-BR daemon guide, `develop-feature` skill and this ledger. SDK/Core/Registry contracts, toolchain, master and license are unchanged. No commit, push, PR, vendor binary execution, raw capture, firmware action or external evidence upload occurred.

## Fase 3 audit session: profile JSON import/export

The user asked to audit the master document's **Fase 3 ("Daemon, IPC e perfis")** against the implementation. The audit confirmed the Fase 3 acceptance criteria are met: the user-session daemon runs, the typed versioned IPC client answers all listed operations (`health`, `list_devices`, `get_device`, `get_capabilities`, `get_profile`, `save_profile`, `set_lighting`, connection events/status), profiles persist in SQLite with schema-versioned JSON, duplicate daemons are refused, and the integration tests pass. One master §4.6 item was only library-level: profiles supported `to_json`/`from_json` but there was **no user-facing import/export command** (recorded as a limitation in ADR 0004, `daemon.md` and the profiles section above). That gap was closed in this session.

### Actual delivery

| Component | State | Evidence and boundary |
| --- | --- | --- |
| Profile file import/export | Completed (concluído) | New `peripheral_hub_daemon::{export_profile, import_profile}` helpers wrap `Profile::to_json`/`from_json` and local file I/O only; the file path is a client argument, never a wire field |
| Diagnostic CLI | Completed (concluído) | `profile-export ID [PATH]` (stdout when `PATH` omitted) and `profile-import PATH` on the daemon client; import validates before `SaveProfile`, so unknown versions/malformed files are rejected, not stored |
| Automated tests | Completed (concluído) | New `apps/daemon/tests/profile_files.rs` (2 tests): export→import→repository round-trip and rejection of future-version/malformed/missing files. Suite grew 69 → **71 passed / 0 failed / 0 ignored** |

No IPC wire change was needed: export uses `GetProfile`, import uses `SaveProfile`, both already in the v2 contract. No new dependency was added (`std::fs` only).

### Commands and actual outcomes

| Command / action | Outcome |
| --- | --- |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0 |
| `cargo check --workspace --locked` | Passed, exit 0 |
| `cargo test --workspace --locked` | Passed: **71 passed / 0 failed / 0 ignored** (+2 from `profile_files.rs`) |
| `cargo build --workspace --locked` | Passed, exit 0 |
| Real daemon/client smoke (`serve --simulate` on a private temp socket + profile DB) | Passed: `profile-save gaming Gaming 255 0 0` → `profile-export gaming FILE` wrote the `profile_schema_version` 1 document → `profile-delete` → `profile-import FILE` restored it → `profile-get gaming` returned the same RGB; a bumped `profile_schema_version` was rejected with "unsupported profile schema version" (exit 1); SIGTERM removed the socket |

The smoke test used only the explicit software simulator and a temporary local file/socket; no physical device, vendor binary, capture or external service was involved.

### Limitations

1. **File path is client-side:** import/export read/write a local path chosen by the caller; the daemon never receives a path over IPC and there is no file picker (the desktop keeps its own client-side import/export in TypeScript).
2. **No data migration beyond forward-only:** only `profile_schema_version` 1 exists, so no re-encode of older stored rows is implemented; a future version bump would need an explicit migration rather than the current reject-on-read behavior.
3. **Model-driven application still pending:** applying a saved profile to a connected device remains unimplemented; profiles record intent only. *(Superseded by the profile-application session below.)*
4. **Windows evidence remains static:** the Windows target check/Clippy are compilation evidence only, not runtime/DACL validation.

### Relevant files

Added `apps/daemon/src/profile_files.rs` and `apps/daemon/tests/profile_files.rs`; updated `apps/daemon/src/{lib.rs,main.rs}`. Documentation: `daemon.md`, pt-BR `users/daemon.md`, [ADR 0004](decisions/0004-storage-and-profiles.md) and this ledger. IPC protocol, SDK/Core/Registry contracts, storage/profiles libraries, toolchain, master and license are unchanged. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

## Declarative driver system (user "Fase 4": native and declarative drivers)

The user asked to implement the driver system and called it "Fase 4". In the master document the declarative/native driver work is §4.4 (part of Fase 2), while Fase 4 is the user interface, which is already implemented and validated (desktop section above). This execution therefore implements the master's §4.4 "drivers nativos e declarativos" without changing the SDK 1.0/Core/Registry contracts, and does not touch the desktop. Phase 1–3, storage/profiles, desktop and the profile import/export records above are preserved historical evidence. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| Declarative crate | Completed (concluído) | New member `peripheral-hub-driver-declarative`: typed TOML manifest model (`manifest_version`, `model`, `match`, `capabilities`, `operations`), semantic validation and a bounded report encoder. Reuses SDK `DeviceMatchRule`/`Capabilities`; SDK contracts unchanged |
| Bounded interpreter | Completed (concluído) | `DeclarativeFactory`/`DeclarativeDriver`: open validates transport limits, `execute` gates on capabilities, enforces a rate limit, writes exactly one output/feature report of 1..64 bytes within 0..30 s, and requires the full write count. `input` reports, empty/oversized payloads, overlapping offsets, undeclared zones and unmapped effects are rejected |
| Explicit provenance | Completed (concluído) | `DeclarativeFactory::new(manifest, mode)` takes the mode from the daemon, not the transport; the daemon builds it `Simulated` under `--simulate` and `Physical` otherwise, and the existing manager rejects a mode mismatch instead of reporting simulated success |
| JSON Schema | Completed (concluído) | Real `schemas/device-manifest.schema.json` (draft 2020-12) mirroring the typed model; the Rust validator stays authoritative for overlap/effect/capability rules the schema cannot express |
| Fictitious examples | Completed (concluído) | `examples/declarative-driver/manifest.toml` (synthetic `0xF0F0:0x0001`) and the compiled `examples/native-driver` crate (`0xF0F1:0x0002`); both are labeled fictitious, are not runtime support and are not registered by the daemon on their own |
| Daemon loading | Completed (concluído) | New `register_manifest_dir` (`serve --manifests DIR`): sorted paths, fail-fast on the first invalid/conflicting manifest, no runtime fetch; `--manifests` is rejected outside `serve` |
| Tests | Completed (concluído) | Crate unit tests (validation/encoding) + integration tests (identification, capability gating, rate limit, non-matching descriptor) + native example test + daemon `tests/declarative.rs` (load, list, connect, capabilities, valid + invalid `SetLighting`, invalid-manifest abort). Suite grew 71 → **84 passed / 0 failed / 0 ignored** |
| Documentation | Completed (concluído) | [ADR 0006](decisions/0006-declarative-drivers.md); updated [driver-system](driver-system.md), [SDK](sdk.md), [daemon](daemon.md) and this ledger; `develop-driver`/`add-device`/`validate-device` skills updated |

`toml` was added as a workspace dependency; `serde`, `thiserror` and `tracing` were already present. Added workspace members: `crates/driver-declarative`, `examples/native-driver`. The physical registry is still empty and no proprietary protocol, packet, capture or hardware compatibility is claimed.

### Commands and actual outcomes

Validation reused the temporary Rust 1.99.0/Zig tools (`RUSTUP_HOME=target/phase3-rustup`, `CARGO_HOME=target/phase2-cargo`, linker shim `target/phase3-cc`).

| Command / action | Outcome |
| --- | --- |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no target restrictions or warning suppressions |
| `cargo check --workspace` | Passed, exit 0; ten members including the two new ones |
| `cargo test --workspace` | Passed: **84 passed / 0 failed / 0 ignored** (declarative unit 7, declarative integration 3, native example 1, daemon declarative 2; +13) |
| `python3 -c "import json;json.load(open('schemas/device-manifest.schema.json'))"` | Passed; the schema file parses as JSON |

The tests use synthetic descriptors and in-memory transports only; no physical device was enumerated, opened or written.

### Failures corrected and design review

- The first declarative duplicate-zone test built an incomplete manifest (an operation with no report), so it failed as a TOML error instead of the intended semantic `Invalid`. The test now duplicates a complete operation and asserts the duplicate-zone rejection.
- The daemon `manifests.rs` initially used `thiserror::Error` without the crate being a daemon dependency; `thiserror` was added to `apps/daemon/Cargo.toml`. No behavior changed.
- The manifest `FieldSource` enum needed `PartialOrd`/`Ord` for the duplicate-source `BTreeSet`; the derive was extended.

### Limitations and pending priorities

1. **Volatile lighting only:** the interpreter models only `SetLighting`. Displays, macros, feature/input reads, persistence and firmware need a reviewed native crate or a future contract revision, not a manifest.
2. **Manifest matching needs real evidence:** a manifest matches only with VID/PID plus interface or HID usage evidence, but the hypothetical `0xF0F0:0x0001` identity is synthetic. A real `devices/` entry still requires authorized protocol evidence and `validated-on-hardware` results.
3. **No runtime loading:** manifests load only from an explicit `--manifests` directory at startup; there is no hot-reload or dynamic plugin loading.
4. **Windows evidence is static:** as before, the Windows target checks are compilation evidence only.
5. **Later scope/governance:** DRA, packaging/release, real hardware operations, CI, private reporting channel and explicit license confirmation remain pending.

### Relevant files and decisions

Added `crates/driver-declarative/{Cargo.toml,src/,tests/declarative.rs}`, `examples/declarative-driver/manifest.toml`, `examples/native-driver/{Cargo.toml,src/,tests/}`, `schemas/device-manifest.schema.json`, `apps/daemon/src/manifests.rs`, `apps/daemon/tests/declarative.rs` and [ADR 0006](decisions/0006-declarative-drivers.md). Updated root `Cargo.toml`/`Cargo.lock`, `apps/daemon/{Cargo.toml,src/lib.rs,src/main.rs}`, the [driver system](driver-system.md)/[SDK](sdk.md)/[daemon](daemon.md) references, this ledger and the `develop-driver`/`add-device`/`validate-device` skills. SDK/Core/Registry contracts, toolchain, master and license are unchanged. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

## Domain features §4.6: profile application (user "Fase 5")

The user asked to start "Fase 5 — RGB, macros, displays e aplicação de perfis". In the master document Fase 5 is the out-of-scope DRA; the requested substance is §4.6 "Recursos de domínio". After explicit confirmation, this execution implements the first domain increment, **profile application**, and leaves RGB (already modeled), macros and displays for later increments. Phase 1–3, storage/profiles, desktop and the driver-system records above are preserved historical evidence. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| IPC contract | Completed (concluído) | `IPC_VERSION` raised 2 → 3 with `Operation::ApplyProfile { interface_id, profile_id }`; replies with the existing `Reply::Outcome`. Older clients are rejected by the incompatible-version path |
| Daemon manager | Completed (concluído) | `ApplyProfile` loads the profile, enforces the optional `device_model` binding (`NotSupported` on mismatch), requires an open driver, and reuses the extracted `execute_lighting` capability-gated path shared with `SetLighting`; a profile with no `lighting` returns `NotSupported`. No raw report or new device operation |
| Daemon CLI | Completed (concluído) | `profile-apply ID INTERFACE_ID` plus help text |
| Desktop | Completed (concluído) | `src/types/ipc.ts` bumped to `IPC_VERSION = 3` with the new operation; `ipcClient.applyProfile` and `DaemonController.applyProfile`; the Profiles screen gained an apply target listing connected devices, a per-row Apply action, and reports the actual mode |
| Tests | Completed (concluído) | Daemon `tests/manager.rs` adds one end-to-end test (success, unbound apply, model mismatch, no-lighting, not-connected) → Rust **84 → 85 passed**; frontend `ipcClient`, `daemonController` and `screens` tests added → **53 → 57 passed** |
| Documentation | Completed (concluído) | New [ADR 0007](decisions/0007-profile-application.md); updated [IPC](ipc.md), [daemon](daemon.md), [desktop](desktop.md), [ADR 0004](decisions/0004-storage-and-profiles.md) and this ledger |

SDK 1.0/Core/Registry contracts, the storage/profiles libraries and the profile schema version are unchanged. The physical registry remains empty and no hardware compatibility is claimed.

### Commands and actual outcomes

Validation reused the temporary Rust 1.99.0/Zig tools (`RUSTUP_HOME=target/phase3-rustup`, `CARGO_HOME=target/phase2-cargo`, linker shim `target/phase3-cc`) and the local Node/pnpm under `target/phase4-node`.

| Command / action | Outcome |
| --- | --- |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no target restrictions or warning suppressions |
| `cargo check --workspace --locked` | Passed, exit 0 |
| `cargo test --workspace --locked` | Passed: **85 passed / 0 failed / 0 ignored** (+1 daemon `manager`) |
| `pnpm --filter @peripheral-hub/desktop typecheck` | Passed, exit 0 |
| `pnpm --filter @peripheral-hub/desktop test` | Passed: **57 passed / 0 failed** across 9 files (+2 ipcClient/controller, +2 screens) |

All tests use the explicit software simulator, an in-memory profile store and local IPC doubles; no physical device was enumerated, opened or written.

### Failures corrected and design review

- The `SetLighting` execution body was duplicated logic; it was extracted into a private `execute_lighting` helper so `ApplyProfile` cannot drift from the capability/mode checks. No behavior changed.

### Limitations and pending priorities

1. **Apply needs an open driver:** `ApplyProfile` targets an already connected interface; it does not auto-connect or auto-discover. The desktop surface lists connected devices for that reason.
2. **Volatile lighting only:** a profile can only carry a lighting preference today, so applying performs no display/macro action. A profile with no applicable preference is rejected, not silently accepted.
3. **Remaining §4.6 work:** richer RGB (multi-zone/effects), macro persistence/limits in the daemon, and the display contract/preview remain pending.
4. **Later scope/governance:** DRA, packaging/release, real hardware operations, CI, private reporting channel and explicit license confirmation remain pending.
5. **Windows evidence is static:** as before, the Windows target checks are compilation evidence only.

### Relevant files and decisions

Updated `crates/ipc/src/protocol.rs`, `apps/daemon/src/{manager.rs,main.rs}`, `apps/daemon/tests/manager.rs`, `apps/desktop/src/types/ipc.ts`, `apps/desktop/src/services/{ipcClient.ts,daemonController.ts}`, `apps/desktop/src/features/profiles/ProfilesScreen.tsx`, `apps/desktop/src/services/{ipcClient.test.ts,daemonController.test.ts}` and `apps/desktop/src/features/screens.test.tsx`; added [ADR 0007](decisions/0007-profile-application.md) and updated the [IPC](ipc.md)/[daemon](daemon.md)/[desktop](desktop.md) references and [ADR 0004](decisions/0004-storage-and-profiles.md). SDK/Core/Registry contracts, toolchain, master and license are unchanged. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

## Domain features §4.6: generic multi-zone RGB lighting (continuation of user "Fase 5")

After profile application, the user asked to proceed with the next §4.6 increment only: a generic RGB lighting system (their "Fase 5"; the master's own Fase 5 is the out-of-scope DRA). Multi-zone lighting was the explicit remaining §4.6 gap recorded above. It reuses the SDK lighting DTOs, the capability-gated path shared with profiles, and the declarative interpreter, without inventing any proprietary protocol. Macros and displays were left untouched, and no commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| IPC contract | Completed (concluído) | `IPC_VERSION` raised 3 → 4 with `Operation::SetLightingScene { interface_id, scene }` and `LightingScene { commands }`; `MAX_SCENE_ZONES = 256`; `LightingScene::validate()` rejects empty, oversized and duplicate-zone scenes before transport |
| Daemon manager | Completed (concluído) | New `execute_scene` pre-validates all commands against the driver capabilities before any write, then executes in order; `ApplyProfile` now routes its resolved command through the same path; `operation_error` returns the typed `Fault` |
| Declarative driver | Completed (concluído) | Rate limit trackers are now per zone (`BTreeMap<String, Instant>`, cleared on open/close); the example manifest declares a second `logo` zone so multi-zone scenes are exercised end to end |
| Daemon CLI | Completed (concluído) | `lighting-scene INTERFACE_ID ZONE:R:G:B [...]` plus help text |
| Desktop | Completed (concluído) | `src/types/ipc.ts` bumped to `IPC_VERSION = 4` with `LightingScene`; `ipcClient.setLightingScene`/`DaemonController.setLightingScene`; the Lighting screen selects one or more declared zones and applies them as a single scene, with a clear-all/select-all toggle and an "Apply to N zones" action |
| Tests | Completed (concluído) | IPC `codec` shape test; daemon `manager` capability-gate test (valid, empty/duplicate/oversized shape, undeclared zone/brightness/effect, non-open interface); daemon `declarative` multi-zone scene test; declarative crate per-zone independence assertion → Rust **85 → 88 passed**; frontend `set_lighting_scene` parameter test → **57 → 58 passed** |
| Documentation | Completed (concluído) | New [ADR 0008](decisions/0008-generic-multi-zone-lighting.md); updated [IPC](ipc.md), [daemon](daemon.md), [desktop](desktop.md), [driver system](driver-system.md) and this ledger |

SDK 1.0/Core/Registry contracts, the storage/profiles libraries, the profile schema version and the declarative manifest schema are unchanged. The physical registry remains empty and no hardware compatibility is claimed.

### Commands and actual outcomes

Validation reused the temporary Rust 1.99.0/Zig tools (`RUSTUP_HOME=target/phase3-rustup`, `CARGO_HOME=target/phase2-cargo`, linker shim `target/phase3-cc`) and the local Node/pnpm under `target/phase4-node`.

| Command / action | Outcome |
| --- | --- |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no target restrictions or warning suppressions |
| `cargo check --workspace --locked` | Passed, exit 0 |
| `cargo test --workspace --locked` | Passed: **88 passed / 0 failed / 0 ignored** (+1 ipc, +1 daemon manager, +1 daemon declarative) |
| `pnpm --filter @peripheral-hub/desktop typecheck` | Passed, exit 0 |
| `pnpm --filter @peripheral-hub/desktop test` | Passed: **58 passed / 0 failed** across 9 files (+1 ipcClient) |
| `pnpm --filter @peripheral-hub/desktop build` | Passed, exit 0 (Vite production build) |

All tests use the explicit software simulator, an in-memory profile store, the fictitious declarative example and local IPC doubles; no physical device was enumerated, opened or written.

### Failures corrected and design review

- The declarative interpreter tracked a single `last_sent: Option<Instant>` for the whole driver, so a multi-zone scene written back-to-back was rejected as a rate violation and could tear down the session. It now tracks one timestamp per zone, matching the one-report-per-operation model, so independent zones no longer interfere.

### Limitations and pending priorities

1. **Declared zones only:** a scene can only target zones the driver declares; an undeclared zone fails the whole scene with `InvalidArgument`/`NotSupported` rather than being ignored.
2. **Volatile lighting only:** scenes are still bounded volatile-lighting writes; effects remain the SDK's fixed enum and no persistent-memory or firmware operation exists.
3. **Simulated evidence:** all new paths are exercised against the simulator and the fictitious declarative example; no physical device validated them.
4. **Remaining §4.6 work:** macro persistence/limits in the daemon and the display contract/preview remain pending.
5. **Later scope/governance:** DRA, packaging/release, real hardware operations, CI, private reporting channel and explicit license confirmation remain pending.
6. **Windows evidence is static:** as before, the Windows target checks are compilation evidence only.

### Relevant files and decisions

Updated `crates/ipc/src/protocol.rs`, `crates/ipc/tests/codec.rs`, `crates/driver-declarative/src/driver.rs`, `crates/driver-declarative/tests/declarative.rs`, `apps/daemon/src/{manager.rs,main.rs}`, `apps/daemon/tests/{manager.rs,declarative.rs}`, `examples/declarative-driver/manifest.toml`, `apps/desktop/src/types/ipc.ts`, `apps/desktop/src/services/{ipcClient.ts,daemonController.ts,ipcClient.test.ts}` and `apps/desktop/src/features/lighting/LightingScreen.tsx`; added [ADR 0008](decisions/0008-generic-multi-zone-lighting.md) and updated the [IPC](ipc.md)/[daemon](daemon.md)/[desktop](desktop.md)/[driver system](driver-system.md) references. SDK/Core/Registry contracts, toolchain, master and license are unchanged. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

## pnpm configuration correction (Etapa 1)

The user asked to fix two pnpm configuration points. The audit found the repository already satisfied both, so this step confirmed the current state and corrected stale documentation instead of changing configuration. **No `pnpm-workspace.yaml`/`package.json` change was required.**

| Item | State | Evidence and boundary |
| --- | --- | --- |
| esbuild build approval | Already correct (correto) | `pnpm-workspace.yaml` uses `allowBuilds: {esbuild: true}`, the current pnpm 10 syntax (`renamedKeys` maps `allowBuilds` → `onlyBuiltDependencies`); no legacy `pnpm.onlyBuiltDependencies` key remains in any config |
| `minimumReleaseAge` | Already configured (configurado) | `pnpm-workspace.yaml` sets `minimumReleaseAge: 1440` |
| Documentation | Corrected (corrigido) | Two stale ledger rows that named `pnpm.onlyBuiltDependencies` and one that claimed `minimumReleaseAge` was unset were corrected above |

### Commands and actual outcomes

| Command / action | Outcome |
| --- | --- |
| `pnpm install --frozen-lockfile` (pnpm 10.34.7, `CI=true`) | Passed; lockfile up to date, 101 packages, esbuild postinstall ran |
| `pnpm --dir apps/desktop typecheck` | Passed, exit 0 |
| `pnpm --dir apps/desktop test` | Passed: **66 passed / 0 failed** across 9 files |
| `pnpm --dir apps/desktop build` | Passed, exit 0 (Vite production build) |

## Domain features §4.6: macro persistence and editing (continuation of user "Fase 5")

After generic multi-zone lighting, the user asked to proceed with the next §4.6 increment only: a persistent, editable macro system (their "Fase 5"; the master's own Fase 5 is the out-of-scope DRA). Macros are modeled as a bounded, versioned, validated action sequence that is **persisted and edited but never executed** — this increment implements storage, CRUD, IPC, daemon and desktop editing only; no input injection, hooking or automatic run-on-load exists. Displays were left untouched and no commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| Macro crate | Completed (concluído) | New `crates/macros` (`peripheral-hub-macros`) with `MacroId`, `KeyPress`, `MacroAction`, `MacroRepeat`, `Macro`, `MacroSummary`, `MacroFile`; bounded limits (`MAX_MACRO_ACTIONS = 4096`, `MAX_MACRO_NAME = 64`, `MAX_DELAY_MS = 60000`, `MAX_MACRO_REPEAT = 1000`, `MAX_MACRO_INTERVAL_MS = 60000`, `MAX_MACRO_ITERATION_MS = 600000`, `MAX_MACRO_TOTAL_MS = 3600000`); `validate()` rejects malformed names, non-token keys, empty/oversized action lists, out-of-range repeats/pauses and over-long iteration/total durations before any write |
| Storage | Completed (concluído) | `MacroRepository` over the storage crate's SQLite abstractions, in its own forward-only migration stream (`MACRO_SCHEMA_VERSION = 1`) and a **separate** `macros.sqlite` file, because the storage `Database` tracks a single global `user_version` that the profile stream already owns |
| IPC contract | Completed (concluído) | `IPC_VERSION` raised 4 → 5 with `ListMacros`, `GetMacro`, `SaveMacro`, `DeleteMacro`, `Reply::Macros`/`Reply::Macro` and `Fault::Macro(MacroFault)` (`Invalid`, `UnsupportedVersion`, `NotFound`, `Storage`); the `SaveMacro` payload carries the validated `Macro` (wire key `macro`) |
| Daemon manager | Completed (concluído) | `DeviceManager::with_stores` accepts explicit profile and macro repositories; `SaveMacro` validates the payload and, when a profile link is present, confirms the referenced profile exists (else `Fault::Macro(Invalid)`); persistence never touches a device or executes the macro |
| Daemon CLI | Completed (concluído) | `macros`, `macro-get ID`, `macro-save ID NAME KEY [KEY ...]`, `macro-delete ID`, `macro-export ID [PATH]`, `macro-import PATH`, plus a `--macros PATH` serve flag (default under the per-user data directory) and updated help |
| Desktop | Completed (concluído) | `src/types/ipc.ts` bumped to `IPC_VERSION = 5` with the macro DTOs; `ipcClient`/`DaemonController` macro methods; the Macros screen lists stored macros, edits/repeats/profile link, saves/deletes via IPC, and keeps the descriptive dry-run simulation with an explicit "execution is not implemented" notice |
| Tests | Completed (concluído) | Macro crate unit tests (ids, validation, bounds, JSON round-trip, invalid/future import, CRUD, file-backed persistence, reject-before-write); IPC `codec` round-trip and sanitized macro-fault test; daemon `manager` CRUD/validation/profile-link test; daemon `macro_files` export/import test; daemon `ipc_roundtrip` macro test → Rust **88 → 102 passed**; frontend macro-model, client/controller and screen tests → **58 → 66 passed** |
| Documentation | Completed (concluído) | New ADR 0009; updated [IPC](ipc.md), [daemon](daemon.md), [desktop](desktop.md) and this ledger |

SDK 1.0/Core/Registry contracts, the storage/profiles libraries, the profile schema version and the declarative manifest schema are unchanged. The physical registry remains empty and no hardware compatibility is claimed.

### Commands and actual outcomes

Validation reused the temporary Rust 1.99.0/Zig tools (`RUSTUP_HOME=target/phase3-rustup`, `CARGO_HOME=target/phase2-cargo`, linker shim `target/phase3-cc`) and the local Node/pnpm under `target/phase4-node`.

| Command / action | Outcome |
| --- | --- |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no target restrictions or warning suppressions |
| `cargo check --workspace --locked` | Passed, exit 0 |
| `cargo test --workspace --locked` | Passed: **102 passed / 0 failed / 0 ignored** (+8 macros, +2 daemon macro_files, +1 daemon manager, +2 ipc codec, +1 daemon ipc_roundtrip) |
| `pnpm --dir apps/desktop typecheck` | Passed, exit 0 |
| `pnpm --dir apps/desktop test` | Passed: **66 passed / 0 failed** across 9 files (+3 macroModel, +3 ipcClient/controller, +1 daemonStore, +2 screens) |
| `pnpm --dir apps/desktop build` | Passed, exit 0 (Vite production build) |

All tests use the explicit software simulator, in-memory or temporary-file macro stores and local IPC doubles; no physical device was enumerated, opened or written, and no macro was executed.

### Failures corrected and design review

- The `macro` wire/file key cannot be a Rust field name (it is a reserved keyword), so the field is `macro_value` with `#[serde(rename = "macro")]`, keeping the wire/file contract (`macro`) consistent with the profile envelope.
- Macros cannot share the profile SQLite file: the storage `Database` tracks a single global `pragma user_version` and both features wanted migration `v1`. Macros therefore use a separate `macros.sqlite` file and their own migration stream.
- The frontend `Fault` union gained a `macro` variant; the `errors.ts` exhaustive switch was extended rather than left non-total (a missing case failed `tsc`).

### Limitations and pending priorities

1. **No execution:** stored macros are never run, injected or hooked. `SaveMacro`/`macro-import` validate and persist only; the CLI and screen say so explicitly. A future execution path would require a separate, reviewed driver/session integration and is out of scope here.
2. **No button binding:** association to physical buttons is not modeled; the only optional link is an advisory reference to a stored **profile** id (`Macro.profile`), which the daemon checks for existence but does not act on.
3. **Symbolic keys only:** actions reference symbolic key names; no captured or replayed input exists, so keystroke logging is not possible.
4. **Simulated evidence:** all paths are exercised against the simulator and local IPC doubles; no physical device validated them.
5. **Remaining §4.6 work:** the display contract/preview remains pending.
6. **Later scope/governance:** DRA, packaging/release, real hardware operations, CI, private reporting channel and explicit license confirmation remain pending; Windows evidence stays static.

### Relevant files and decisions

Added `crates/macros/{Cargo.toml,src/{lib,error,model,migrations,repository,tests}.rs}`, `apps/daemon/src/macro_files.rs`, `apps/daemon/tests/macro_files.rs` and [ADR 0009](decisions/0009-macro-persistence.md). Updated root `Cargo.toml`, `crates/ipc/{Cargo.toml,src/protocol.rs}`, `crates/ipc/tests/codec.rs`, `apps/daemon/{Cargo.toml,src/{lib.rs,manager.rs,main.rs},tests/{manager.rs,ipc_roundtrip.rs}}`, `apps/desktop/src/{types/ipc.ts,stores/daemonStore.ts,services/{errors.ts,ipcClient.ts,daemonController.ts,ipcClient.test.ts,daemonController.test.ts},stores/daemonStore.test.ts,features/macros/{macroModel.ts,MacrosScreen.tsx,macroModel.test.ts},features/screens.test.tsx}` and the [IPC](ipc.md)/[daemon](daemon.md)/[desktop](desktop.md) references. SDK/Core/Registry contracts, toolchain, master and license are unchanged. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.

## Domain features §4.6: display contract and local preview (completion of the requested "Fase 5")

After macros, the user asked to implement the display part, the last §4.6 domain surface. The SDK already models `DisplayCapabilities` but has no display/frame command, and the master forbids sending content to a physical screen without a validated protocol. This increment is therefore deliberately **contract validation plus a local preview only**: no SDK/IPC change, no frame encoder, no transport write and no hardware operation. The fictitious simulator now declares a bounded display so the screen can be exercised in simulation.

### Actual delivery and evidence levels

| Component | State | Evidence and boundary |
| --- | --- | --- |
| Display model | Completed (concluído) | New `apps/desktop/src/features/displays/displayModel.ts`: `bytesPerPixel` (3 for `rgb888`, 2 for `rgb565`), `frameBytes`, `validateFrame` (undeclared format and `max_frame_bytes` overflow) and `previewFrame` (solid/checker/gradient CSS background plus a descriptive "not transmitted" label); no encoder or transport call exists |
| Display screen | Completed (concluído) | `DisplaysScreen` keeps the declared capability table and adds format/pattern/RGB controls, byte computation, contract errors and a preview constrained to the declared aspect ratio; it states that the preview is local and that effective control requires a validated driver protocol |
| Simulator | Completed (concluído) | `simulation:peripheral` declares 64×32, `rgb888`/`rgb565`, 30 fps and 6144 max frame bytes; a daemon `GetCapabilities` test asserts the declaration, and no display command is executed because SDK 1.0 has none |
| IPC/SDK | Unchanged (inalterado) | Reuses the existing `GetCapabilities`; `IPC_VERSION` stays 5, `SDK_CONTRACT_VERSION` stays 1.0 and no `DeviceCommand` variant was added |
| Tests | Completed (concluído) | Frontend `displayModel` tests (byte math, format/limit rejection, preview construction/colour) and a connected-display screen test → **66 → 75 passed across 10 files**; daemon display-contract test → Rust **102 → 103 passed** |
| Documentation | Completed (concluído) | New [ADR 0010](decisions/0010-display-contract-preview.md); updated [desktop](desktop.md) and this ledger; ADR 0009 now points to the display increment |

### Commands and actual outcomes

Validation reused the temporary Rust 1.99.0/Zig tools (`RUSTUP_HOME=target/phase3-rustup`, `CARGO_HOME=target/phase2-cargo`, linker shim `target/phase3-cc`) and the local Node/pnpm under `target/phase4-node`.

| Command / action | Outcome |
| --- | --- |
| `cargo fmt --all`; `cargo fmt --all --check` | Passed, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0; no target restrictions or warning suppressions |
| `cargo check --workspace --locked` | Passed, exit 0 |
| `cargo test --workspace --locked` | Passed: **103 passed / 0 failed / 0 ignored** (+1 daemon display-contract test) |
| `pnpm --dir apps/desktop typecheck` | Passed, exit 0 |
| `pnpm --dir apps/desktop test` | Passed: **75 passed / 0 failed** across 10 files (+8 displayModel, +1 screens) |
| `pnpm --dir apps/desktop build` | Passed, exit 0 (Vite production build) |

All tests use the explicit software simulator and local IPC doubles; no physical device was enumerated, opened or written, and no frame was encoded or transmitted.

### Limitations and pending priorities

1. **No transmission:** the preview is a local CSS background. There is no frame buffer, encoder, rate loop, packet layout or transport write, and the screen says so.
2. **No SDK extension:** a real display path first needs a reviewed SDK contract extension and an evidence-backed protocol; neither is authorized or implemented.
3. **Fictitious declaration:** the 64×32 display belongs to the explicit `simulation:peripheral` model and is not evidence of any real product's display.
4. **No hardware evidence:** the display contract is exercised only through capability IPC and desktop tests; no physical screen was controlled.
5. **§4.6 coverage:** profiles, lighting, macros and displays now exist at the editing/persistence/contract/preview level; real device behavior remains a later, separately authorized stage.
6. **Later scope/governance:** DRA, packaging/release, real hardware operations, CI, private reporting channel and explicit license confirmation remain pending; Windows evidence stays static.

### Relevant files and decisions

Added `apps/desktop/src/features/displays/displayModel.ts`, `apps/desktop/src/features/displays/displayModel.test.ts` and [ADR 0010](decisions/0010-display-contract-preview.md). Updated `apps/daemon/src/simulation.rs`, `apps/daemon/tests/manager.rs`, `apps/desktop/src/features/displays/DisplaysScreen.tsx`, `apps/desktop/src/features/screens.test.tsx`, [desktop](desktop.md), this ledger and [ADR 0009](decisions/0009-macro-persistence.md). SDK/Core/Registry contracts, the IPC version, the macro/profile schemas, toolchain, master and license are unchanged. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.
