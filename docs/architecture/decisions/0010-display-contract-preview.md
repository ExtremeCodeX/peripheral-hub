# ADR 0010: Display contract and local preview without frame transmission

- Status: implemented as software (desktop contract validation and local CSS preview over the existing `GetCapabilities` operation); simulated evidence only, no frame encoder, no transport write, no physical or hardware-compatibility claim.
- Date: 2026-10-20.

## Context

The master document's §4.6 lists displays as a domain feature and §6 asks for a display "panel of information and contract/preview", with effective controls only in a driver with validated support. The SDK already models `DisplayCapabilities` (dimensions, `PixelFormat` list, max frame rate and max frame bytes) and validates those bounds, and the daemon returns them through `GetCapabilities`. However, SDK 1.0's `DeviceCommand` has no display/frame variant; adding one would change the frozen SDK contract. Sending pixels to a physical screen also requires a discovered, evidence-backed protocol, which does not exist yet. The remaining §4.6 gap recorded after macros was the "display contract/preview".

## Decisions

1. Keep SDK 1.0/Core/Registry and the wire contract unchanged. There is no new IPC operation, no `DeviceCommand` variant, no transport call and no frame encoder for this increment. The existing `GetCapabilities` operation supplies the contract.
2. Implement the display increment entirely in the presentation layer: `displayModel.ts` mirrors the SDK bounds (three bytes for `rgb888`, two for `rgb565`), computes uncompressed frame size from the declared dimensions, and validates a prospective frame against the declared formats and `max_frame_bytes` before anything is shown.
3. `DisplaysScreen` keeps the declared-capability table and adds an explicitly local preview: format, pattern (solid/checker/gradient) and RGB colour produce a CSS background constrained to the declared aspect ratio. The preview is rendered in this window only; the screen states that nothing is encoded or transmitted and that effective control requires a validated driver protocol.
4. Make the contract exercisable in simulation without claiming a real device: the fictitious `simulation:peripheral` driver declares a bounded 64×32 display (`rgb888`/`rgb565`, 30 fps, 6144 max frame bytes). A daemon integration test asserts this declaration; no display command is executed because none exists.
5. Record the increment in the desktop reference and status ledger, and add this ADR. No commit, push, PR, vendor execution, hardware access, capture or external data sharing is performed.

## Consequences

The display contract is now visible, bounded and testable end to end (declared capabilities → IPC → desktop validation/preview), with unit tests for byte math, format/limit rejection and preview construction, a daemon test for the simulated declaration, and screen render coverage. **Frame transmission remains deliberately absent:** there is no encoder, no packet format, no rate loop and no device write, so the preview must not be read as display control. A future implementation needs a reviewed SDK contract extension, an evidence-backed protocol and explicit hardware authorization. With this increment the §4.6 domain surface (profiles, lighting, macros, displays) is covered at the editing/persistence/contract level; real device behavior is still limited to the software simulator. See the [desktop reference](../desktop.md), [IPC reference](../ipc.md) and [status ledger](../implementation-status.md).
