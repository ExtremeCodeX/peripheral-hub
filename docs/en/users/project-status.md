# User guide: current availability

[Português brasileiro](../../pt-BR/users/project-status.md)

Peripheral Hub has a runnable Phase 2 Rust daemon/diagnostic CLI, real HID enumeration/transport implementation and explicit software simulation. There is no installable desktop GUI, physical driver, profiles, macros, displays or DRA yet. Linux daemon/software evidence and Windows static checks do not validate physical control or Windows runtime compatibility. See the [daemon guide](../../architecture/daemon.md).

## Installation and usage

There is no desktop installer or profile file format. The actual daemon command is `cargo run -p peripheral-hub-daemon -- serve --simulate`; query with `health` and `list` in another terminal. Without `--simulate`, the daemon performs real discovery with no registered physical control drivers. Remaining desktop/DRA/config/schema files are placeholders, not functional packages or compatibility entries.

When real releases exist, this guide must be expanded with tested installation, separate desktop/daemon startup, supported profile import/export, diagnostics, and an evidenced compatibility matrix. Windows 10/11 is the intended first target, not a current compatibility guarantee.

## Troubleshooting and support requests

If you are looking for device support now, record the model, OS, firmware if known, and desired capabilities without publishing a serial, private path, executable, or raw capture. See [adding a device](../contributors/adding-a-device.md). GitHub templates and security reporting contacts are not yet configured; never submit sensitive evidence to a public issue.

Consult the [README](../../../README.md) and [implementation status](../../architecture/implementation-status.md) for actual progress. Rust libraries and the daemon can be built/tested; use the [daemon guide](../../architecture/daemon.md) for diagnosis. There is no desktop GUI to launch yet.
