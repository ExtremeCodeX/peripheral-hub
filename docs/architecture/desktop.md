# Desktop interface (Tauri 2 + React/TypeScript)

**Status: frontend, restricted bridge and native shell implemented; the shell compiles and runs on a host with the webview dependencies installed, but not in the sandbox used for automated validation.** This document follows the [master specification](../../OPENCODE_IMPLEMENTATION_MASTER.md) Fase 4 and [ADR 0005](decisions/0005-desktop-ui.md). See the [daemon guide](daemon.md), [IPC reference](ipc.md) and [validation ledger](implementation-status.md).

## Boundary and data flow

```text
apps/desktop/src            React/TypeScript UI (no USB/HID, no socket)
        ↓ invoke("daemon_call", { operation })
apps/desktop/src-tauri      single Tauri command, typed Operation → Response
        ↓ Endpoint::default_for_user(), 5 s client deadline
peripheral-hub-ipc          same-user local framed IPC (IPC version 5)
        ↓
peripheral-hub-daemon       lifecycle, profiles/storage, registry, drivers
```

The UI submits a typed `Operation` and receives the daemon's versioned `Response`. There is no other frontend capability: no filesystem, shell or network plugin is granted (see `src-tauri/capabilities/default.json`), and the CSP restricts sources to self plus the Tauri IPC origin.

## Frontend modules

| Module | Responsibility |
| --- | --- |
| `src/types/ipc.ts` | Faithful mirror of `crates/ipc/src/protocol.rs` and the SDK DTOs it re-uses (`IPC_VERSION = 5`, adjacently tagged enums) |
| `src/services/transport.ts`, `tauriTransport.ts` | `DaemonTransport` interface and the Tauri `invoke` implementation |
| `src/services/ipcClient.ts` | Version check, `Ok`/`Err` unwrapping, reply-discriminant assertions |
| `src/services/errors.ts` | `DaemonFaultError`, `BridgeError`, fault/device-error descriptions, unsupported/permission helpers |
| `src/services/viewModel.ts` | Pure presentation helpers (titles, VID:PID, capability rows, counts) |
| `src/services/daemonController.ts` | Operation orchestration and error mapping |
| `src/stores/daemonStore.ts`, `src/hooks/useDaemon.tsx` | Observable store and provider/hook with an injectable transport |
| `src/features/*` | Device list/details, lighting, profiles, macros, displays, diagnostics screens |
| `src/features/profiles/profileFile.ts`, `src/features/macros/macroModel.ts` | Versioned profile and macro JSON plus validation/simulation; the macro model mirrors the IPC macro types (actions, finite repeat, advisory profile link) |

## Screens and UX states

- **Devices:** discovered devices, connection status, product/manufacturer/VID:PID, matched model, and simulation-vs-physical mode. Unknown models are shown as read-only and are never opened.
- **Device details:** explicit capability rows (including "not declared"), connection controls, and support/partial-support warnings.
- **Lighting:** color/brightness/effect controls only when the daemon reports the capability; one or more declared zones can be selected and applied together as a scene (`set_lighting_scene`), so a multi-zone device updates in a single request. Success and error messages come from the daemon; a simulated operation is labeled as simulated.
- **Profiles:** create/rename/edit/save/apply/delete over the real profile operations, plus client-side versioned JSON import/export. Applying targets an explicitly selected connected device, honors the profile's optional model binding, and reports the actual `simulated`/`physical` provenance. Invalid data is rejected with typed faults.
- **Macros:** list stored macros and create/rename/edit/save/delete them over the real macro operations, with a finite repeat and an optional advisory profile link, plus client-side versioned JSON import/export and a descriptive simulation trace. Storing or importing never executes a macro, injects input or hooks global input; the screen says so explicitly.
- **Displays:** declared display contract and a local contract preview. The screen validates a prospective frame against the declared dimensions, format and byte limit, and renders a solid/checker/gradient preview in the browser. Nothing is encoded or transmitted; effective control is only for a driver with validated display support, which no driver implements yet.
- **Diagnostics:** daemon version, SDK contract, IPC version, discovery state, transport status, and DRA/support-request guidance.

Empty, loading, disconnected, permission-denied, error and unsupported states are rendered explicitly. Forms validate before sending and navigation is keyboard-accessible.

## Native dependencies

The React frontend only needs Node 20+ and pnpm. Building and running the Tauri shell additionally needs the platform webview and a C toolchain; the shell is intentionally excluded from the root Cargo workspace so `cargo check --workspace` works without them.

| Platform | Packages |
| --- | --- |
| Arch Linux | `base-devel`, `pkgconf`, `gtk3`, `webkit2gtk-4.1` (`librsvg` only when bundling icons) |
| Debian/Ubuntu | `build-essential`, `pkg-config`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libssl-dev` |
| Windows | MSVC C++ build tools and the WebView2 runtime (Windows 10/11) |

A host without these fails inside the `gtk-sys`/`webkit2gtk-sys` build scripts (missing `pkg-config` package `gtk+-3.0`/`webkit2gtk-4.1`), not in this project's own code.

## Building and validating

```sh
pnpm install --frozen-lockfile
pnpm --filter @peripheral-hub/desktop typecheck   # tsc --noEmit
pnpm --filter @peripheral-hub/desktop test         # vitest run
pnpm --filter @peripheral-hub/desktop build        # tsc --noEmit && vite build
```

The React UI needs a reachable daemon at runtime: start `cargo run -p peripheral-hub-daemon -- serve --simulate` first. The shell has no `build.devUrl`, so both debug and release embed `frontendDist` (`../dist`) and need no dev server or Node runtime. Tauri decides dev vs. production by the `tauri` crate's `custom-protocol` feature — not by the Cargo profile — and `features = []` does not enable it, so `cfg(dev)` is always on; without `devUrl` the assets are still embedded and loaded over `tauri://localhost`. Editing `tauri.conf.json` alone may not recompile the crate, so force it with `touch src/main.rs` after changing the config. Build the frontend once, then run the shell:

```sh
pnpm --filter @peripheral-hub/desktop build
cargo run --manifest-path apps/desktop/src-tauri/Cargo.toml
```

It requires platform webview dependencies and is intentionally excluded from the root Cargo workspace. `bundle.icon` points at `apps/desktop/src-tauri/icons/*.png`; `tauri::generate_context!` fails to compile if the configured icon files are missing. For Vite live-reload development, add `"devUrl": "http://localhost:5173"` and `"beforeDevCommand": "pnpm dev"` back under `build`, then start `pnpm dev` before the shell.

## Evidence and limits

- Automated evidence here: TypeScript typechecking, 75 Vitest tests across 10 files, a production Vite build, and `cargo fmt`/`clippy -D warnings`/`check`/`test`/`build --locked` on the Rust workspace.
- The Tauri shell is not compiled or launched in this sandbox (`pkg-config`, a C toolchain, GTK and `webkit2gtk` are absent). On a host with the native dependencies above it compiles with `cargo run --manifest-path apps/desktop/src-tauri/Cargo.toml` and, against a live `serve --simulate` daemon, lists the simulated device, applies a simulated lighting change and persists a profile across a daemon restart.
- There is no installer, no single-instance behaviour and no daemon auto-start; the daemon and desktop are started manually. Windows remains static-evidence only.
- Macros are persisted and edited, not executed. Displays expose the declared contract and a local preview; no frame is encoded or transmitted, and the simulator's display declaration is fictitious. No physical control capability or input injection is claimed by this layer.
