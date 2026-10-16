# ADR 0004: Versioned SQLite storage and profile persistence

- Status: implemented in the storage/profiles phase; the desktop binding and profile JSON import/export were added later in [ADR 0005](0005-desktop-ui.md); import/export as a daemon CLI command and encryption remain pending.
- Date: 2026-10-15.

## Context

The daemon and IPC existed without persistence, and profile operations were explicit `NotSupported` placeholders. This decision adds the master document's Fase 3 ("Daemon, IPC e perfis") without inventing a profile protocol or a desktop layer.

## Decisions

1. Add two implemented members, `peripheral-hub-storage` and `peripheral-hub-profiles`, preserving SDK 1.0, Core and Registry contracts. No desktop, DRA, declarative interpreter or CI package is added here.
2. Keep one owned SQLite connection per `Database` (no pool). All persistence runs on the serialized daemon manager worker, so connection contention is not modeled. `foreign_keys` is enabled and a 5-second `busy_timeout` is set.
3. Track schema state with SQLite `PRAGMA user_version` and a caller-supplied, forward-only `Migration` list. Migrations must start at version 1, be strictly increasing and unique, run each in its own transaction, and reject a database whose stored version is newer than the code rather than downgrading it.
4. Keep profile payloads as a validated, schema-versioned JSON document (`PROFILE_SCHEMA_VERSION`, currently 1) independent from the SDK, manifest and IPC versions. Persist both a normalized `schema_version` column and the serialized payload; on read, reject unknown versions and re-validate rather than guessing.
5. Store the entire validated profile in a single `profiles` row keyed by `ProfileId` (insert-or-replace). No relational over-modeling of lighting fields. Lighting is the existing SDK `LightingCommand`; the profile carries no raw packets.
6. Extend IPC to `IPC_VERSION = 2` with typed `ListProfiles`, `GetProfile`, `SaveProfile` and `DeleteProfile` operations, and a `Fault::Profile(ProfileFault)` variant (`Invalid`, `UnsupportedVersion`, `NotFound`, `Storage`). The version bump means older clients/languages are rejected by the existing incompatible-version path rather than silently misparsed.
7. Default persistence lives in a per-user data path under `PERIPHERAL_HUB_DATA`, else `XDG_DATA_HOME`/`HOME/.local/share` (or `LOCALAPPDATA`) as `peripheral-hub/profiles.sqlite`; `--profiles PATH` overrides it. The daemon reuses the same bounded manager deadline for profile operations and returns typed faults, never success on failure.

## Consequences

Profiles are durable and validated across restart with real SQLite migrations on the existing same-user trust boundary. The storage crate is reusable but intentionally minimal; only the profile repository is wired today. No import/export CLI, no file locking beyond SQLite, no encryption, no migration tooling beyond forward-only `user_version`, and no schema for other future data is claimed. A single connection means concurrent multi-process writers to the same file are not a supported model. `ProfileFault` is a deliberate part of the v2 wire contract and must not be renumbered casually.

The desktop binding and client-side profile JSON import/export were implemented in [ADR 0005](0005-desktop-ui.md); a daemon-side import/export command, device-model-driven application of profiles, and encryption/sanitization of stored data remain explicit follow-ups. See [daemon](../daemon.md), [IPC](../ipc.md), [desktop](../desktop.md), [ADR 0003](0003-hid-daemon-local-ipc.md) and the [status ledger](../implementation-status.md).
