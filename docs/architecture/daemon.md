# Daemon, HID transport, profiles and diagnostic client

## Delivered boundaries

The workspace adds `peripheral-hub-transport-hid`, `peripheral-hub-ipc`, `peripheral-hub-storage`, `peripheral-hub-profiles`, and `peripheral-hub-daemon` to the three Phase 1 libraries. SDK 1.0 and Core/Registry source contracts are preserved. The daemon owns lifecycle state; a serialized 32-item manager queue keeps blocking enumeration/driver operations off async IPC tasks. Tokio provides local socket/pipe I/O, deadlines, bounded clients, polling and signal shutdown.

Profile persistence uses the validated SQLite repository from `peripheral-hub-storage`/`peripheral-hub-profiles` (see [ADR 0004](decisions/0004-storage-and-profiles.md)). The store is edited through typed IPC only; there is no raw SQL, arbitrary query or file path in the wire contract.

The physical registry is empty: no proprietary protocol is known or registered. Real HID interfaces remain visible as unknown/discovered. Opening is explicit and requires a matching reviewed driver and a source-provided policy; the default physical source refuses unknown opening. No raw packet IPC/CLI exists.

## HID behavior and limitations

`HidDiscovery` uses `hidapi` enumeration/refresh without sending reports. Descriptors retain observed VID/PID, optional interface/usage, manufacturer/product and private path/serial; firmware remains unknown. IPC summaries omit locators and serials. Native Linux uses `linux-native-basic-udev`; Windows uses `windows-native`. Linux does not require system libudev development headers with this selected backend. macOS transport builds are not configured or validated.

Observed endpoint paths receive session-local opaque IDs with a random source epoch. Removal/reappearance gets fresh tokens; a daemon restart cannot reuse stale interface tokens. HIDAPI may enumerate several usage collections for one path: duplicates collapse into one endpoint and conflicting usages become unknown. **Physical parent grouping across different paths is not implemented.** Rather than merge devices by VID/PID or possibly duplicated serials, discovery conservatively assigns separate unit tokens. Thus counts are interfaces, not validated physical unit counts, and composite/dongle grouping is incomplete. Polling also cannot guarantee detection of transitions occurring entirely between scans.

`HidDiscovery::open_reviewed()` verifies a still-observed descriptor and an explicit `HidPolicy`: byte/deadline bounds, input numbering, allowed output/feature IDs, and a minimum operation interval. A physical driver must provide proven lengths/framing and safe payloads. Report-ID allowlisting alone does not establish payload safety. No report is sent merely by opening.

One bounded worker owns each HID handle. Input uses real `read_timeout`; writes and feature requests call real `hidapi` APIs. IDs are normalized separately from payload, including ID zero. Invalid directions, report IDs, payload lengths, rates, responses and short/invalid write counts fail; OS/backend errors are sanitized. `CheckedTransport` remains the SDK outer guard.

The caller stops waiting at its deadline and retires a timed-out handle; **hidapi cannot cancel every in-flight kernel feature/write operation**. Such a request can complete later at the OS, and must never be replayed or declared successful. There is no guarantee that an already-started write was cancelled. At most one worker/queued operation per opened handle is retained; a stalled kernel call may keep that worker alive until the OS returns or the process exits. Feature writes expose only hidapi's success/error indication, not an independently verified payload count. Windows-native output-count/padding conventions still need hardware-specific validation; unexpected counts fail closed.

## Lifecycle

Polling defaults to one second. Valid snapshots flow through unchanged Core validation; removed or changed descriptors close/retire old handles and emit events. Unknown interfaces are not probed. Explicit `connect` constructs a uniquely matched driver, opens a reviewed transport and validates capabilities; `disconnect` closes it. Transport, permission, protocol and timeout failures transition the handle to failed and prevent fictitious state updates. Connection events form a bounded journal, not an unlimited push stream.

Enumeration failure retains last-known Core records internally but returns an error for list/open/driver requests. `health` exposes the sanitized failure without converting it into an empty successful inventory or enabling simulation. Expired queued mutations are discarded before execution; mutations already executing when a caller disconnects/times out can have an unknown outcome. Clients never replay them automatically.

## Profile persistence

Profiles are model-agnostic, schema-versioned `Profile` records: a stable `ProfileId`, a bounded name, an optional bound `DeviceId` model and an optional typed `LightingCommand`. `SaveProfile` validates and upserts a row; `ListProfiles` returns summaries; `GetProfile`/`DeleteProfile` operate by id and report `NotFound` rather than succeeding silently. Reads re-validate stored data and reject unknown schema versions. The database is a single per-user SQLite file (default `peripheral-hub/profiles.sqlite`; override with `--profiles PATH`). Migration state uses `PRAGMA user_version` with forward-only migrations; a newer on-disk schema is refused, not downgraded. Profiles record intent only: saving or listing them does not open a device or send any report.

## Explicit simulator

Only `serve --simulate` registers the fictitious `simulation:peripheral` model. Its synthetic VID/PID, descriptors, JSON memory reports and lighting state are not a vendor protocol or compatibility entry. All summaries/state/outcomes identify `simulated`. The real daemon never falls back to this source after HID failure. Simulation supports controlled presence changes, capability validation, lighting/state, reconnects, and injected transport errors in tests. It performs no global input injection or hardware writes.

## Declarative drivers

`serve --manifests DIR` loads reviewed declarative TOML manifests from a directory the operator names, via `register_manifest_dir`. Paths are sorted, each manifest is parsed and semantically validated by `peripheral-hub-driver-declarative`, and startup aborts on the first invalid or conflicting manifest; nothing is fetched or loaded at runtime. The factory's provenance is `Simulated` under `--simulate` and `Physical` otherwise, and the manager rejects any driver outcome whose mode differs from the source mode, so a declarative report is never mislabeled as physical. Loading requires a matching model: a manifest only matches when its `DeviceMatchRule` evidence (VID/PID plus interface or HID usage) agrees with the observed descriptor, and the interpreter supports volatile `SetLighting` for declared zones only. See the [driver system](driver-system.md), `examples/declarative-driver/manifest.toml` and [ADR 0006](decisions/0006-declarative-drivers.md).

## Running

Install pinned Rust 1.99.0 and a native platform linker. From the repository root:

```sh
cargo build --workspace --locked
cargo run -p peripheral-hub-daemon -- --help
```

Terminal 1 (safe software-only development):

```sh
cargo run -p peripheral-hub-daemon -- serve --simulate
```

Terminal 2:

```sh
cargo run -p peripheral-hub-daemon -- health
cargo run -p peripheral-hub-daemon -- list
cargo run -p peripheral-hub-daemon -- events
```

Copy an actual `interface_id` from `list` and replace `INTERFACE_ID` below; this is a placeholder, not a fixed supported-device token:

```sh
cargo run -p peripheral-hub-daemon -- connect INTERFACE_ID
cargo run -p peripheral-hub-daemon -- capabilities INTERFACE_ID
cargo run -p peripheral-hub-daemon -- lighting INTERFACE_ID 255 0 0
cargo run -p peripheral-hub-daemon -- state INTERFACE_ID
cargo run -p peripheral-hub-daemon -- disconnect INTERFACE_ID
cargo run -p peripheral-hub-daemon -- simulate-absent
cargo run -p peripheral-hub-daemon -- simulate-present
```

Profiles persist across daemon restarts in the default per-user database (`--profiles PATH` overrides it, on both `serve` and the client commands):

```sh
cargo run -p peripheral-hub-daemon -- profile-save gaming Gaming 255 0 0
cargo run -p peripheral-hub-daemon -- profiles
cargo run -p peripheral-hub-daemon -- profile-get gaming
cargo run -p peripheral-hub-daemon -- profile-delete gaming
```

Import/export use the schema-versioned JSON envelope (`profile_schema_version`). `profile-export ID [PATH]` writes the validated document (stdout when `PATH` is omitted); `profile-import PATH` parses and validates the file before `SaveProfile`, so an unknown version or malformed document is rejected rather than stored:

```sh
cargo run -p peripheral-hub-daemon -- profile-export gaming gaming.json
cargo run -p peripheral-hub-daemon -- profile-import gaming.json
```

The file path is a client-side argument only; it never crosses the IPC boundary and profiles never carry raw reports or vendor payloads.

For non-destructive physical enumeration: `cargo run -p peripheral-hub-daemon -- enumerate`. For a physical discovery daemon: `cargo run -p peripheral-hub-daemon -- serve`. No physical lighting driver is bundled, so real device commands are unsupported. This enumeration command does not validate I/O or device compatibility.

To exercise declarative manifest loading (validates and registers the fictitious example; the synthetic `0xF0F0:0x0001` device is only produced by a matching test source, so no real hardware is implied):

```sh
cargo run -p peripheral-hub-daemon -- serve --simulate --manifests examples/declarative-driver
```

`--manifests` is only valid with `serve`; an invalid or conflicting manifest aborts startup.

The daemon/client choose the same per-user endpoint by default. `--endpoint PATH` overrides it for each process; Unix parent directories must be owned by the effective user and mode 0700, while Windows custom names must retain the validated local prefix and current SID suffix. No administrator elevation is requested. Ctrl+C (and Unix SIGTERM) stops the daemon and closes handles; a second server on the same endpoint is refused. Run only one default endpoint per user; custom endpoints are explicit separate development instances.

The desktop frontend and its restricted Tauri bridge are implemented as source under `apps/desktop` (see the [desktop reference](desktop.md)); the Tauri shell is not built or launched in this environment, and no DRA launch command exists yet. SQLite-backed profiles, their IPC operations and schema-versioned JSON import/export are implemented; model-driven application of a profile to a device remains outside this execution. Declarative manifests load only from an explicit `--manifests` directory and execute only bounded volatile-lighting operations. Exact IPC framing and authorization are in [IPC reference](ipc.md), decisions in [ADR 0003](decisions/0003-hid-daemon-local-ipc.md), [ADR 0004](decisions/0004-storage-and-profiles.md), [ADR 0005](decisions/0005-desktop-ui.md) and [ADR 0006](decisions/0006-declarative-drivers.md), and actual tests/platform evidence in the [ledger](implementation-status.md).
