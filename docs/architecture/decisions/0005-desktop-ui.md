# ADR 0005: Restricted Tauri desktop over the local IPC contract

- Status: implemented as software (React/TypeScript frontend and a restricted Tauri bridge source); the Tauri shell is not built or run in this environment and no packaging is claimed.
- Date: 2026-10-16.

## Context

The daemon, IPC version 2 and SQLite-backed profiles existed without a user interface. This decision adds the master document's Fase 4 ("Interface de usuário") without inventing a device protocol, a second frontend framework, or direct USB/HID access from the UI. Acceptance required the desktop to start and operate against the real daemon with the simulated driver, including profiles.

## Decisions

1. Add the frontend under `apps/desktop` as a pnpm workspace member: Vite, React 18 and strict TypeScript. No second frontend framework, no router dependency (navigation is local component state), and no Electron.
2. Keep all daemon access behind one restricted Tauri command, `daemon_call(Operation) -> Response`. The frontend builds typed `Operation` values and never opens USB/HID, the socket, or an arbitrary network request. The bridge uses `Endpoint::default_for_user()` and the same five-second client deadline as the daemon CLI.
3. Mirror the wire contract in `src/types/ipc.ts` (`IPC_VERSION = 2`, adjacently tagged enums). `IpcClient` verifies the version, unwraps `Ok`/`Err`, converts a `Fault` to a typed `DaemonFaultError`, and asserts each reply discriminant; transport failures are a separate `BridgeError`, never success.
4. Give the shell its own standalone Cargo manifest and keep it **out** of the root Rust workspace (`exclude` in the root `Cargo.toml`). Building it needs platform webview dependencies (for example webkit2gtk/gtk on Linux, WebView2 on Windows); excluding it keeps `cargo check --workspace` green on hosts without a webview.
5. Drive UI state from a small observable store plus `useSyncExternalStore`, with `DaemonController` performing the calls. Business logic (view models, profile file, macro model) stays in pure TypeScript modules so it is testable without a browser or a socket.
6. Treat macro support as editing, validation, versioned persistence and a descriptive simulation trace only. The MVP does not hook global input or inject keystrokes, and it must not imply that a remapping reached hardware.
7. Present every capability explicitly, including not-declared ones, and label simulated versus physical mode and unknown versus matched models so the UI never implies physical control that was not validated.

## Consequences

The presentation layer can be typechecked, unit-tested and production-built with pnpm alone, and its pure logic is covered by Vitest tests. The Tauri bridge is real source but unverified here: without webview/GTK dependencies the shell is not compiled, so there is no desktop runtime, installer, or end-to-end GUI evidence in this environment. Windows remains static-evidence only. Secrets, raw captures and proprietary artifacts are never bundled; there is no automatic external transmission.

DRA, manifest/interchange schemas, CI, packaging, signed releases, real drivers and model-driven application of a profile to a device remain explicit follow-ups. See the [desktop reference](../desktop.md), [IPC reference](../ipc.md), [ADR 0004](0004-storage-and-profiles.md) and the [status ledger](../implementation-status.md).
