# Implementation status and evidence ledger

Last updated: 2026-10-13. English is the technical reference. This ledger distinguishes delivered documentation from working software.

## Authorized scope

The current user requests the first **documentary and organizational foundation only**. The broader implementation authorization in the master specification does not expand this request. No software, workspaces, schemas, drivers, CI jobs, or runtime/MCP configuration are implemented in this session.

## Baseline audit

- Read all 571 lines of `OPENCODE_IMPLEMENTATION_MASTER.md`, existing `AGENTS.md`, all seven `SKILL.md` files, architecture placeholders, manifests, representative source files, and directory inventories. There were no nested instruction files or existing ADRs in the inventory.
- Read-only recursive inventory: **55 files and 92 directories**. The master specification was the only nonempty file. All other 54 files were empty; existing organization was preserved.
- Master specification SHA-256: `ee537731a3aa19adca1bf5d81ec2fe05b9be7bc12b88a730ee0bbdb5e43ac2af`.
- Rust/Node manifests and lockfiles, Tauri configs, schema files, source placeholders, workflows, templates, CODEOWNERS, and LICENSE had no content.
- The checkout is not a Git repository. No canonical remote, commit baseline, or Git diff is available.
- Python 3.13.15 is available for local documentary checks. `rustc`, `cargo`, `node`, `pnpm`, `opencode`, and `tshark` were not found on PATH. No REA/USBPcap installation or hardware access was verified; no hardware operations were attempted.

## Foundation delivery

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

## Subsequent implementation milestones (outside this stage)

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

READMEs, contribution/security priorities, device contribution, and availability guidance have pt-BR coverage. Architecture, ADRs, protocol/research procedures, and skills remain English technical references. Detailed installation, usage, profiles, compatible-device catalog, and troubleshooting manuals wait for real functionality; no empty guide is presented as complete. `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, and `LICENSE` remain pre-existing empty files with explicit governance/license follow-up above.
