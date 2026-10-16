# User guide: current availability

[Português brasileiro](../../pt-BR/users/project-status.md)

Peripheral Hub has a runnable Rust daemon/diagnostic CLI, real HID enumeration/transport implementation, explicit software simulation, SQLite-backed profile persistence with typed profile IPC and JSON import/export, a bounded declarative driver interpreter (`serve --manifests DIR`, `schemas/device-manifest.schema.json`, volatile lighting only), and a desktop React/TypeScript frontend with a restricted Tauri bridge. There is no physical driver, no built/installable desktop shell in the current environment, and no macros injection, display transmission or DRA yet. Linux daemon/software evidence and Windows static checks do not validate physical control or Windows runtime compatibility. See the [driver system](../../architecture/driver-system.md), [daemon guide](../../architecture/daemon.md) and [desktop reference](../../architecture/desktop.md).

## Installation and usage

There is no desktop installer or shipped profile file format. The actual daemon command is `cargo run -p peripheral-hub-daemon -- serve --simulate`; query with `health` and `list` in another terminal. Without `--simulate`, the daemon performs real discovery with no registered physical control drivers.

The desktop frontend typechecks, unit-tests and builds with pnpm (`pnpm install`, then `pnpm --filter @peripheral-hub/desktop typecheck|test|build`) and expects a running daemon. The Tauri shell under `apps/desktop/src-tauri` is real source but is not compiled or launched in this environment because platform webview dependencies are unavailable, so there is no installable desktop app yet. The declarative interpreter and `schemas/device-manifest.schema.json` are implemented and cover volatile lighting only; the remaining DRA/schema/config files are placeholders, not functional packages or compatibility entries.

When real releases exist, this guide must be expanded with tested installation, separate desktop/daemon startup, supported profile import/export, diagnostics, and an evidenced compatibility matrix. Windows 10/11 is the intended first target, not a current compatibility guarantee.

## Troubleshooting and support requests

If you are looking for device support now, record the model, OS, firmware if known, and desired capabilities without publishing a serial, private path, executable, or raw capture. See [adding a device](../contributors/adding-a-device.md). GitHub templates and security reporting contacts are not yet configured; never submit sensitive evidence to a public issue.

Consult the [README](../../../README.md) and [implementation status](../../architecture/implementation-status.md) for actual progress. Rust libraries and the daemon can be built/tested; use the [daemon guide](../../architecture/daemon.md) and [desktop reference](../../architecture/desktop.md) for diagnosis. There is no built desktop GUI to launch in this environment.
