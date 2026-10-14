# Peripheral Hub AI skills

These procedures follow [AGENTS.md](../../AGENTS.md), the [master specification](../../OPENCODE_IMPLEMENTATION_MASTER.md), and the [status ledger](../../docs/architecture/implementation-status.md). Current scope authorizes Phase 2 HID/daemon/IPC/lifecycle and explicit simulation, preserving Phase 1 contracts. Phase 3 desktop requires later review; a skill does not grant hardware execution permission.

## Choosing a skill

| Skill | Trigger and purpose |
| --- | --- |
| [add-device](add-device/SKILL.md) | Coordinate an end-to-end device contribution and identify missing evidence |
| [reverse-engineer](reverse-engineer/SKILL.md) | Inspect authorized vendor artifacts; separate observations from hypotheses |
| [capture-protocol](capture-protocol/SKILL.md) | Plan authorized captures, correlate actions, and sanitize exports |
| [develop-driver](develop-driver/SKILL.md) | Build support from evidence using implemented SDK/validators |
| [validate-device](validate-device/SKILL.md) | Check identity, contracts, negative paths, and physical support claims |
| [develop-feature](develop-feature/SKILL.md) | Change application layers within their architectural boundaries |
| [review-contribution](review-contribution/SKILL.md) | Review safety, evidence, compatibility, documentation, and actual check results |

The device workflow is identification → evidence → implementation → validation → human review. Load the skill relevant to the present step; supporting skills may be read as needed without launching subagents. Human-only contributors can follow the same procedures.

## Loading and prerequisites

Each `.agents/skills/<name>/SKILL.md` begins with YAML `name`, `description`, and `compatibility`. Read it and any linked references before acting. OpenCode's documented skill location is preserved; see [official skills guidance](https://opencode.ai/docs/skills/). Restart OpenCode after changing skills/configuration so a new session loads them.

The [SDK reference](../../docs/architecture/sdk.md) and [daemon guide](../../docs/architecture/daemon.md) document actual APIs/commands. Run root fmt, Clippy, check and tests; use `cargo run -p peripheral-hub-daemon -- serve --simulate` with diagnostic `health`/`list` for safe end-to-end development. `enumerate` is real non-destructive HID inventory, not DRA or hardware control validation. DRA, physical drivers, scaffold/manifest validators, profiles, frontend and CI remain absent. REA is optional and unconfigured; do not invent its/DRA commands.

## Evidence and handoff

Record source and scope, exact checks, limits, capability evidence, and blockers. Use `simulated`, `inferred`, `tested`, `validated-on-hardware` for support claims; use `confirmed-on-hardware`, `inferred`, `unverified` for protocol hypotheses. These label sets serve different purposes and do not imply automatic promotion.

Store raw research locally under ignored storage. Export only reviewed, authorized, synthetic/sanitized artifacts. Every skill must stop before missing authorization, unsafe/unknown hardware operations, invented evidence, or unsupported tooling. A skill grants neither permission to execute vendor binaries nor permission to publish a PR.
