# Adding a device, with or without AI

[Português brasileiro](../../pt-BR/contributors/adding-a-device.md)

**Current stage:** [SDK 1.0, Registry, and Core](../../architecture/sdk.md) are preserved; [Phase 2](../../architecture/daemon.md) implements real HID discovery/report transport, protected IPC, daemon and explicit simulation. DRA, manifest validators, scaffold tools and physical drivers remain absent. A real driver needs authorized scope, proven policy/protocol and capability-specific hardware validation, not just successful enumeration.

## Start with identity and a limited goal

Record the actual model, OS, firmware/revision, transport, VID/PID, interfaces, usage page/usage, and additional distinguishing evidence. Do not identify a protocol from VID/PID alone or expose identifying serials/private paths publicly. Select a specific capability, such as a known safe volatile color change, rather than promising full support.

Check existing entries first. `devices/manufacturer/device-model/` is an empty structural placeholder, not a supported example.

## Collect evidence responsibly

Read the [security policy](../../../SECURITY.md) and [research workflow](../../reverse-engineering/research-workflow.md). Obtain consent, record lawful vendor software provenance/version/hash, and keep raw binaries/captures in ignored local storage. Optional static REA analysis can inform hypotheses, but physical captures/tests may be necessary.

Dynamic execution and capture elevation require explicit authorization and a suitable laboratory. Correlate baseline and individual safe vendor-UI actions with target-interface traffic. Do not replay unknown packets or attempt firmware/bootloader/persistent writes. Sanitize and review any exported evidence.

## Document before implementing

Use the [protocol format](../../protocols/specification-format.md) to record evidenced reports, initialization, sequencing, ACKs, checksums, limits, uncertainty, and revision differences. Protocol hypotheses use `confirmed-on-hardware`, `inferred`, or `unverified`. Unsupported and unsafe operations must be explicit.

Once SDK/validators exist and code is authorized, choose bounded declarative commands for simple modeled protocols or a reviewed compiled Rust driver for complex ones. Add only confirmed capabilities, truthful errors, and meaningful fake-transport tests. No arbitrary runtime plugins or frontend packet execution.

## Expected future contribution

```text
devices/<manufacturer>/<model>/
  manifest.toml
  protocol.md
  validation.md
  tests/
  fixtures/          # authorized synthetic/sanitized textual data only
  Cargo.toml        # native driver only, with implemented source
  src/              # native driver only
```

Exact manifest fields and schema versions must come from actual implemented models, not this conceptual tree. Fictitious examples belong in `examples/` and cannot enter a validated catalog.

`validation.md` must state OS, firmware/revision, driver revision, tested capability/interface, actual result, and limitations. Distinguish `simulated`, `inferred`, `tested`, and `validated-on-hardware`; software tests cannot imply physical success. Hardware work is opt-in and outside automatic CI.

## With AI

Read [AGENTS.md](../../../AGENTS.md); use [add-device](../../../.agents/skills/add-device/SKILL.md) and the matching evidence/development/validation skills. Skills guide the process but do not grant execution, hardware, or publication permission. Do not upload proprietary evidence to external AI services. Verify commands against real installed tools; DRA commands in the master are proposals today.

## Without AI

Follow the same identity → evidence → implementation → validation → review steps manually, using the linked research and protocol guides. Record unavailable tools and untested capabilities. No OpenCode or REA setup is required to contribute documentation or review evidence.

## Review handoff

Prepare a focused PR proposal with sanitized artifacts, provenance, exact checks/results, and capability-level gaps. Follow [CONTRIBUTING.md](../../../CONTRIBUTING.md) for the existing canonical repository. Explicit license confirmation, private reporting contact, ownership, and active CI still need maintainer action. Agents publish only with explicit authorization.
