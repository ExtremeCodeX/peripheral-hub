# Device lifecycle and identity

**Status: design requirements only.** No enumeration backend or lifecycle implementation exists yet.

## Required stages

1. **Discover:** enumerate observable HID interfaces without sending arbitrary packets. Permission errors and absent backends remain explicit.
2. **Identify:** construct descriptors and physical instance identity; retain interface/usage/revision evidence. A model can have multiple connected physical instances and each instance can expose multiple interfaces.
3. **Match:** ask the registry for candidates, resolve evidence, and refuse ambiguous matches. VID/PID alone does not identify a safe protocol.
4. **Open:** access only the evidenced interface through the transport abstraction. Return explicit permission, disconnected, or transport errors.
5. **Expose capabilities:** declare supported operations and limits for this device/firmware; do not infer full support from one RGB command.
6. **Operate:** validate parameters and capability support, respect sequencing/ACKs/timeouts, and report actual success or error. Unsupported operations use `NotSupported`.
7. **Disconnect/close:** release handles and update state/events; outstanding operations must fail truthfully. A reconnect requires identity and capability re-evaluation, not reuse of a stale handle.

These are conceptual stages, not a finalized state-machine API. Hotplug, duplicate devices, composite interfaces, dongles, and revision changes need meaningful tests when implemented.

## Identity invariants

- Model IDs identify a protocol/driver family; physical instance IDs distinguish connected units.
- A descriptor records what is observed; optional or unknown fields must not be invented.
- Interface, usage page/usage, transport, and additional identification evidence constrain matching.
- Private serials and OS paths may be necessary locally but must be sanitized in public records.
- Enumeration does not prove safe command support, and registry presence does not prove physical validation.

## Validation requirements

Fake-transport tests must exercise duplicate/ambiguous candidates, wrong interfaces, disconnects, errors, and unsupported operations. Optional hardware validation must name OS, firmware/revision, interface, driver revision, capability, observed behavior, and limitations. Automatic CI must not operate physical devices.

See the [device guide](../en/contributors/adding-a-device.md) and [driver system](driver-system.md).
