# Implementation status and evidence ledger

Last updated: 2026-10-16. English is the technical reference. This ledger distinguishes delivered documentation from working software.

## Authorized scope

The current user authorized **Phase 2 — HID communication and daemon** (preserve SDK 1.0/Core/Registry; add real hidapi discovery/transport, same-user local IPC, connection management, explicit simulation and automated tests), then **Fase 3 — storage/profiles** (SQLite persistence and typed profile IPC), then **Fase 4 — desktop interface** (React/Tauri UI over the real daemon). No commit, push or PR is authorized; Phase 5 (DRA), packaging/release and hardware operations remain unauthorized. Phase 1–3 and documentary records below are historical evidence.

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
| pnpm workspace | Completed (concluído) | Root `package.json`, `pnpm-workspace.yaml` and `@peripheral-hub/desktop`; `pnpm-lock.yaml` regenerated (the tracked file was an empty placeholder); esbuild build scripts approved via `pnpm.onlyBuiltDependencies` |
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
| `pnpm install` | First run ignored the empty tracked lockfile; after adding `pnpm.onlyBuiltDependencies`, esbuild 0.21.5/0.25.12 postinstall succeeded and the lockfile was written |
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
| pnpm config audit | `pnpm-workspace.yaml` `allowBuilds: {esbuild: true}` is the current pnpm syntax (maps to `onlyBuiltDependencies`); `package.json` `pnpm.onlyBuiltDependencies` is the older equivalent. No obsolete keys. `minimumReleaseAge` is **not** configured anywhere |
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
3. **Model-driven application still pending:** applying a saved profile to a connected device remains unimplemented; profiles record intent only.
4. **Windows evidence remains static:** the Windows target check/Clippy are compilation evidence only, not runtime/DACL validation.

### Relevant files

Added `apps/daemon/src/profile_files.rs` and `apps/daemon/tests/profile_files.rs`; updated `apps/daemon/src/{lib.rs,main.rs}`. Documentation: `daemon.md`, pt-BR `users/daemon.md`, [ADR 0004](decisions/0004-storage-and-profiles.md) and this ledger. IPC protocol, SDK/Core/Registry contracts, storage/profiles libraries, toolchain, master and license are unchanged. No commit, push, PR, vendor execution, hardware access, capture or external data sharing occurred.
