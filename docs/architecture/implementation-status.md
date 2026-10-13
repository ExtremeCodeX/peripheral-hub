# Implementation status and evidence ledger

Last updated: 2026-10-13. English is the technical reference. This ledger distinguishes delivered documentation from working software.

## Authorized scope

The current user authorizes **Phase 1 — Rust foundation and Device SDK only**: workspace, Core, versioned SDK, identity/capabilities, Registry, errors, structured logging, and unit tests. Phase 2 (HID backend, daemon, IPC, connection monitoring, simulators) requires a later review. The documentary foundation recorded below is historical; its documentation-only restriction no longer describes the active scope.

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
