# ADR 0007: Applying stored profiles to a connected device

- Status: implemented as software (IPC version 3 `ApplyProfile`, daemon manager, diagnostic CLI and desktop apply target); simulated evidence only, no physical or hardware-compatibility claim.
- Date: 2026-10-16.

## Context

[ADR 0004](0004-storage-and-profiles.md) added validated, schema-versioned profiles with `ListProfiles`, `GetProfile`, `SaveProfile` and `DeleteProfile`, but a stored profile could not be applied to a live device. The master document's §4.6 "aplicação de perfis" (the user's "Fase 5" of domain features; the master's own Fase 5 is the out-of-scope DRA) therefore remained an explicit gap. Profiles are the only preference type implemented so far (`lighting: Option<LightingCommand>` plus an optional `device_model` binding), and the SDK/Core/Registry contracts and the profile schema version must not change.

## Decisions

1. Bump the wire contract to `IPC_VERSION = 3` and add `Operation::ApplyProfile { interface_id, profile_id }`. It returns the existing `Reply::Outcome(CommandOutcome)`, so provenance (`simulated`/`physical`) stays explicit and no new reply type or device operation is introduced. Older clients are rejected by the existing incompatible-version path.
2. Reuse the capability-gated lighting path. `ApplyProfile` calls the same private `execute_lighting` helper as `SetLighting`: the manager resolves the open driver, validates the resolved `DeviceCommand::SetLighting` against the driver's declared `Capabilities`, executes it, and rejects any outcome whose mode differs from the source mode. No raw report, byte payload or new transport call is added.
3. Enforce the optional model binding. When `profile.device_model` is `Some(model)`, the target device's matched model must equal it; a mismatch (including an unknown device) returns `NotSupported`. An unbound profile may apply to any device that supports the request.
4. Never fabricate success. A profile with no applicable preference (`lighting: None`) returns `NotSupported`, and an interface that is not open/known returns `NotConnected`/`NotSupported`, rather than a benign empty outcome.
5. Surface it through the existing layers only: the diagnostic CLI gains `profile-apply ID INTERFACE_ID`, and the desktop Profiles screen gains an apply target that lists connected devices and reports the actual mode. The frontend still performs no device/transport access.

## Consequences

The domain's profile-application gap from ADR 0004 is closed and covered by end-to-end tests over the in-memory simulation: success, unbound apply, model-mismatch rejection, no-lighting rejection and not-connected failure. Because only volatile lighting is modeled, applying a profile is still a bounded, non-destructive action; macros and displays remain future domain work and no input injection or frame transmission is introduced. The IPC version bump requires the frontend mirror (`apps/desktop/src/types/ipc.ts`) to move to 3 in lockstep. See the [IPC reference](../ipc.md), [daemon guide](../daemon.md), [desktop reference](../desktop.md), [ADR 0004](0004-storage-and-profiles.md) and the [status ledger](../implementation-status.md).
