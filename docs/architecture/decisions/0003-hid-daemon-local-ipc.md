# ADR 0003: Reviewed HID I/O, serialized daemon and authenticated local IPC

- Status: implemented in Phase 2; hardware I/O and Windows runtime evidence pending.
- Date: 2026-10-13.

## Decisions

1. Preserve SDK 1.0, Core and Registry source contracts. Add only implemented HID, IPC and daemon packages; no desktop, storage or DRA package in this phase.
2. Use hidapi native Linux/basic-udev and Windows-native backends, not a success-returning placeholder. Keep non-destructive enumeration separate from matching, opening and reports. Unknown devices are not opened or written. Parent grouping across paths is deferred rather than guessed from VID/PID/serial.
3. Require explicit reviewed limits/report layouts/IDs/rates before native opening. Normalize report IDs and payload counts. One bounded worker per handle enforces caller deadlines; timed-out handles are retired, while unavoidable in-flight OS cancellation limits remain explicit.
4. Use Tokio for real concurrent IPC/timers/signals, with one serialized blocking manager actor and bounded queues. Poll discovery, close invalidated handles, record lifecycle events and propagate failures. No operation with an expired queued deadline is dispatched.
5. Version IPC separately from SDK and use bounded length-prefixed JSON DTOs. Never expose raw packets. Use same-user Unix ownership/modes/peer credentials and locking; Windows local-only first-instance named pipes have an explicit SID DACL and peer process-owner checks via safe dependencies. Keep forbidden unsafe code in project crates.
6. Include only an explicitly enabled fictitious software simulator. Synthetic JSON report/state behavior is labeled simulated throughout. Physical mode has an empty reviewed registry until real device evidence exists.
7. Use fresh random epochs for source IDs and daemon event cursors. Reconnection must not allow a stale interface token to select a different physical endpoint; mutations are never automatically replayed.

## Consequences

The daemon and diagnostic client run without a GUI and test the actual local IPC/lifecycle path using simulation. Windows code can be checked from Linux, but static checks cannot establish Windows ACL/runtime or physical compatibility. Polling can miss short transitions; grouping remains conservative. Kernel feature/write calls may outlive a deadline, so a timeout is an unknown hardware outcome and retires the handle. No physical protocol, profile persistence, firmware operation, frontend, or distribution support is claimed.

The new native IPC dependencies are justified by safe OS authorization while preserving the unsafe prohibition. Hardware validation, Windows runtime tests and platform-specific permissions documentation remain explicit follow-ups. See [daemon](../daemon.md), [IPC](../ipc.md), and the [status ledger](../implementation-status.md).
