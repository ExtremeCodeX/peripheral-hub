# Local IPC and user-session daemon

**Status: mandatory design direction, not a working protocol.** DTO shapes, framing, message limits, and wire version numbers have not been implemented.

## Boundary and platform policy

The Tauri bridge uses a typed IPC client to request daemon operations. The daemon owns device orchestration and runs in the current user's session; it is not automatically an administrator or privileged system service.

- Windows: named pipes restricted to the same user, with reviewed access-control and peer verification.
- Unix development/future targets: protected Unix domain sockets in an appropriate user-owned runtime location with restrictive permissions and peer verification where available.
- No public network listener or implicit trust based solely on the endpoint's path/name.

Authentication/access restrictions, endpoint ownership, duplicate-instance handling, stale endpoints, cleanup, and OS-specific details must be designed and tested before use. This document does not imply those controls exist.

## Contract requirements

Shared typed DTOs belong in `crates/ipc`. An explicit version and compatibility policy must cover request/response/event formats. Candidate operations required by the specification include `health`, `list_devices`, `get_device`, `get_capabilities`, `get_profile`, `save_profile`, `set_lighting`, and connection/status events. These are operation names to implement, not callable APIs today.

Bound message lengths, input fields, timeouts, and request rates. Use structured errors; unsupported capabilities, permission denial, transport failure, and malformed messages must not become successful responses. The desktop must reflect disconnected and simulated states accurately.

## Runtime requirements

Avoid duplicate daemons in one user session, handle orderly shutdown, reconnect clients, and invalidate stale device state. Permit separate development execution of desktop/daemon once actual packages exist; exact commands are pending real manifests.

Future integration tests should cover version incompatibility, malformed/oversized input, same-user restrictions, duplicate instances, timeouts, disconnects/reconnects, and truthful error propagation. Platform-specific tests must declare their platform; no fabricated cross-platform validation.

See [ADR 0001](decisions/0001-foundation-boundaries.md) and [security architecture](security.md).
