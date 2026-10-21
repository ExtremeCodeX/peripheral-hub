# ADR 0008: Generic multi-zone lighting scenes

- Status: implemented as software (IPC version 4 `SetLightingScene`, daemon pre-validation and CLI, per-zone declarative rate limiting and desktop multi-zone controls); simulated evidence only, no physical or hardware-compatibility claim.
- Date: 2026-10-16.

## Context

The master document's §4.6 lists lighting with color, brightness, **zone** and effect (the user's "Fase 5" of domain features; the master's own Fase 5 is the out-of-scope DRA). The SDK already models zones (`LightingCapabilities.zones`, `LightingCommand.zone`) and drivers declare them, but the application could only send a single `SetLighting` command per request and the declarative interpreter supported exactly one zone. Applying a coordinated multi-zone update — the natural "scene" operation — was not expressible, and the declarative interpreter's rate limit was global per driver, so even a hand-built sequence of per-zone commands could be rejected as if it were duplicate traffic. SDK 1.0/Core/Registry contracts, the profile schema version and the declarative manifest schema must not change.

## Decisions

1. Bump the wire contract to `IPC_VERSION = 4` and add `Operation::SetLightingScene { interface_id, scene }` where `LightingScene { commands: Vec<LightingCommand> }`. It returns the existing `Reply::Outcome`, so provenance stays explicit and no device operation or transport call is added.
2. Bound the scene at the wire: `MAX_SCENE_ZONES = 256`, non-empty, and unique zones, enforced by `LightingScene::validate()` and rejected with `InvalidArgument` before any device access.
3. Pre-validate the whole scene before any I/O. The daemon's `execute_scene` resolves the open driver, then validates **every** command against the driver's declared `Capabilities`; only if all pass does it execute them in order. An unsupported zone, effect or brightness therefore fails the entire request without a partial write, and a rejected scene never tears down the session.
4. Keep the capability-gate shared with profiles: `ApplyProfile` routes its single resolved command through the same `execute_scene` path, so the profile and direct operations cannot drift.
5. Make the declarative interpreter's rate limit **per zone** (`BTreeMap<String, Instant>` keyed by zone, cleared on open/close). Each declared zone maps to its own operation/report with its own `max_rate_per_second`, so a multi-zone scene is not blocked by a different zone's recent write. The declared per-operation rate remains the authority; nothing is replayed or silently dropped.
6. Surface it through existing layers only: the diagnostic CLI gains `lighting-scene INTERFACE_ID ZONE:R:G:B [...]`, and the desktop Lighting screen selects one or more declared zones and applies them as a single scene. The frontend still performs no device/transport access.

## Consequences

Generic multi-zone lighting is now modeled end to end: bounded DTO, atomic capability pre-validation, per-zone declarative rate limiting, CLI and desktop, with unit/integration tests over the in-memory simulation (valid scene, duplicate/oversized/empty shape rejection, undeclared zone and unsupported effect rejection without partial writes, session survival). The IPC version bump requires the frontend mirror (`apps/desktop/src/types/ipc.ts`) to move to 4 in lockstep. Effects remain the SDK's fixed enum and macros/displays stay future domain work; no proprietary protocol, raw report or input injection is introduced. See the [IPC reference](../ipc.md), [daemon guide](../daemon.md), [driver system](../driver-system.md), [desktop reference](../desktop.md) and the [status ledger](../implementation-status.md).
