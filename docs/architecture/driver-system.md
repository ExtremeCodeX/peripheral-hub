# Driver system

**Status: SDK 1.0, explicit factory Registry, and conservative matching are implemented.** No declarative interpreter, production native driver, or physical transport exists. See the [SDK reference](sdk.md).

## Two reviewed support paths

**Declarative drivers:** TOML manifests describe only operations modeled by a project-owned interpreter. Input must pass typed parsing, schema/version checks, and semantic validation. The interpreter must enforce report/payload lengths, parameter types/ranges, timeout/rate limits, and allowed operation types. A manifest cannot become arbitrary code, a shell command, or unrestricted transport access. If execution cannot be safely bounded, the MVP must remain loading/preview-only.

**Native drivers:** complex protocols use Rust crates compiled into the project and explicitly registered. Changes undergo human review and normal software validation. No arbitrary third-party DLLs or runtime plugin loading in the MVP. Add a driver Cargo manifest only alongside real source and valid build integration.

## SDK and registry requirements

Drivers implement SDK 1.0 `DriverFactory`/`DeviceDriver` metadata, construction, opening/closing, capability/state, and typed-command contracts. Registry matching requires VID/PID plus interface/usage evidence and supports exact product/firmware restrictions; ambiguous candidates return an error. Model identity, physical instance and endpoint identities are distinct.

Expose granular capability limits and explicit `NotSupported` behavior. Respect response framing, ACKs, sequencing, timeouts, disconnects, and permission errors. Do not return success when no packet was sent or a transport operation failed. Fake/in-memory transport is a software-test mechanism and must remain visibly separate from physical device behavior.

## Prohibited behavior

No firmware update, bootloader entry, unknown persistent-memory writes, destructive commands, arbitrary frontend packets, or unreviewed external executable loading. Discovery is non-destructive. A protocol hypothesis is not permission to replay an unknown packet.

## Contribution artifacts

Future real entries use `devices/<manufacturer>/<model>/manifest.toml`, `protocol.md`, `validation.md`, `tests/`, and sanitized `fixtures/`. Native entries additionally include implemented `Cargo.toml`/`src/`. Fictitious examples belong under `examples/`, clearly labeled, with no hardware support claim.

The existing `devices/manufacturer/device-model/` files are empty structural placeholders. They are preserved, not catalog entries. Exact manifest fields and schema versions must be designed and implemented later, not guessed here.

Use the [protocol format](../protocols/specification-format.md), [device guide](../en/contributors/adding-a-device.md), and [develop-driver skill](../../.agents/skills/develop-driver/SKILL.md).
