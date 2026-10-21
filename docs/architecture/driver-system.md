# Driver system

**Status: SDK 1.0, Registry, conservative matching, the HID transport and an explicit software simulator are implemented, and the declarative interpreter now exists as `peripheral-hub-driver-declarative` with fictitious examples.** No proprietary physical driver is shipped, and no hardware compatibility is claimed. See [SDK](sdk.md) and [daemon](daemon.md). The physical registry is empty; the fictitious simulator is registered only for `--simulate`, and declarative manifests are registered only via `serve --manifests DIR`.

## Two reviewed support paths

**Declarative drivers:** TOML manifests describe only operations modeled by the project-owned interpreter in `peripheral-hub-driver-declarative`. Input passes typed parsing, a `manifest_version`/SDK-contract check and semantic validation (report kind/length, byte offsets, timeout, rate, capability consistency, effect mapping). The interpreter writes at most one bounded output/feature report per typed `SetLighting` command; it cannot become arbitrary code, a shell command, a firmware/bootloader/persistent-memory operation, or unrestricted transport access. Supported today: volatile lighting for one or more declared zones; the per-operation rate limit is tracked independently per zone, so applying a multi-zone scene is not blocked by another zone's recent write. Anything else returns `NotSupported`. The machine-readable form is `schemas/device-manifest.schema.json`; the Rust validator remains authoritative for rules JSON Schema cannot express. See [ADR 0006](decisions/0006-declarative-drivers.md) and [ADR 0008](decisions/0008-generic-multi-zone-lighting.md).

**Native drivers:** complex protocols use Rust crates compiled into the project and explicitly registered. Changes undergo human review and normal software validation. No arbitrary third-party DLLs or runtime plugin loading. `examples/native-driver` is a fictitious compiled reference, not packaged support.

## Provenance and manifest loading

The interpreter's provenance is explicit, not inferred: `DeclarativeFactory::new(manifest, mode)` takes `Simulated` or `Physical`, and the daemon builds it with `Simulated` under `--simulate` and `Physical` otherwise. The manager rejects any driver outcome whose mode differs from the source mode, so a simulated report can never be presented as physical. `register_manifest_dir` (used by `serve --manifests DIR`) loads manifests explicitly, sorts paths, and aborts startup on the first invalid or conflicting manifest; nothing is fetched or loaded at runtime.

## SDK and registry requirements

Drivers implement SDK 1.0 `DriverFactory`/`DeviceDriver` metadata, construction, opening/closing, capability/state, and typed-command contracts. Registry matching requires VID/PID plus interface/usage evidence and supports exact product/firmware restrictions; ambiguous candidates return an error. Model identity, physical instance and endpoint identities are distinct.

Expose granular capability limits and explicit `NotSupported` behavior. Respect response framing, ACKs, sequencing, timeouts, disconnects, and permission errors. Do not return success when no packet was sent or a transport operation failed. Fake/in-memory transport is a software-test mechanism and must remain visibly separate from physical device behavior.

## Prohibited behavior

No firmware update, bootloader entry, unknown persistent-memory writes, destructive commands, arbitrary frontend packets, or unreviewed external executable loading. Discovery is non-destructive. A protocol hypothesis is not permission to replay an unknown packet. A manifest cannot request any of these: only modeled, bounded volatile-lighting operations are executable.

## Contribution artifacts

Real entries use `devices/<manufacturer>/<model>/manifest.toml`, `protocol.md`, `validation.md`, `tests/`, and sanitized `fixtures/`. Native entries additionally include implemented `Cargo.toml`/`src/`. Fictitious examples belong under `examples/` (`examples/declarative-driver/manifest.toml`, `examples/native-driver/`), clearly labeled, with no hardware support claim.

The existing `devices/manufacturer/device-model/` files remain empty structural placeholders; they are preserved, not catalog entries. A real entry must cite authorized protocol evidence and validated-on-hardware results before any compatibility claim.

Use the [protocol format](../protocols/specification-format.md), [device guide](../en/contributors/adding-a-device.md), and [develop-driver skill](../../.agents/skills/develop-driver/SKILL.md).
