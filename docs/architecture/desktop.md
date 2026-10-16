# Desktop interface (Tauri 2 + React/TypeScript)

**Status: frontend and restricted bridge implemented as source; the Tauri shell is not built or run in the current validation environment.** This document follows the [master specification](../../OPENCODE_IMPLEMENTATION_MASTER.md) Fase 4 and [ADR 0005](decisions/0005-desktop-ui.md). See the [daemon guide](daemon.md), [IPC reference](ipc.md) and [validation ledger](implementation-status.md).

## Boundary and data flow

```text
apps/desktop/src            React/TypeScript UI (no USB/HID, no socket)
        ↓ invoke("daemon_call", { operation })
apps/desktop/src-tauri      single Tauri command, typed Operation → Response
        ↓ Endpoint::default_for_user(), 5 s client deadline
peripheral-hub-ipc          same-user local framed IPC (IPC version 2)
        ↓
peripheral-hub-daemon       lifecycle, profiles/storage, registry, drivers
```

The UI submits a typed `Operation` and receives the daemon's versioned `Response`. There is no other frontend capability: no filesystem, shell or network plugin is granted (see `src-tauri/capabilities/default.json`), and the CSP restricts sources to self plus the Tauri IPC origin.

## Frontend modules

| Module | Responsibility |
| --- | --- |
| `src/types/ipc.ts` | Faithful mirror of `crates/ipc/src/protocol.rs` and the SDK DTOs it re-uses (`IPC_VERSION = 2`, adjacently tagged enums) |
| `src/services/transport.ts`, `tauriTransport.ts` | `DaemonTransport` interface and the Tauri `invoke` implementation |
| `src/services/ipcClient.ts` | Version check, `Ok`/`Err` unwrapping, reply-discriminant assertions |
| `src/services/errors.ts` | `DaemonFaultError`, `BridgeError`, fault/device-error descriptions, unsupported/permission helpers |
| `src/services/viewModel.ts` | Pure presentation helpers (titles, VID:PID, capability rows, counts) |
| `src/services/daemonController.ts` | Operation orchestration and error mapping |
| `src/stores/daemonStore.ts`, `src/hooks/useDaemon.tsx` | Observable store and provider/hook with an injectable transport |
| `src/features/*` | Device list/details, lighting, profiles, macros, displays, diagnostics screens |
| `src/features/profiles/profileFile.ts`, `src/features/macros/macroModel.ts` | Versioned profile and macro JSON plus validation/simulation |

## Screens and UX states

- **Devices:** discovered devices, connection status, product/manufacturer/VID:PID, matched model, and simulation-vs-physical mode. Unknown models are shown as read-only and are never opened.
- **Device details:** explicit capability rows (including "not declared"), connection controls, and support/partial-support warnings.
- **Lighting:** color/brightness/effect controls only when the daemon reports the capability; success and error messages come from the daemon; a simulated operation is labeled as simulated.
- **Profiles:** create/rename/edit/save/apply/delete over the real profile operations, plus client-side versioned JSON import/export. Invalid data is rejected with typed faults.
- **Macros:** editing, validation, versioned persistence and a descriptive simulation trace only; the UI states that no input was injected.
- **Displays:** declared display contract and a placeholder preview; no frame is transmitted, and effective control is only for a driver with validated display support.
- **Diagnostics:** daemon version, SDK contract, IPC version, discovery state, transport status, and DRA/support-request guidance.

Empty, loading, disconnected, permission-denied, error and unsupported states are rendered explicitly. Forms validate before sending and navigation is keyboard-accessible.

## Building and validating

```sh
pnpm install
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

- TypeScript typechecking, Vitest unit tests and a production Vite build run in this environment and are the only desktop evidence recorded.
- The Tauri shell is not compiled or launched here: `pkg-config`, GTK and `webkit2gtk` are absent. There is no desktop runtime, installer or end-to-end GUI evidence, and Windows is static-evidence only.
- Macros are not input injection, and displays are not frame transmission. No physical control capability is claimed by this layer.
