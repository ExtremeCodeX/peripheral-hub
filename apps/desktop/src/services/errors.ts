import type { DeviceError, Fault, ProfileFault } from "../types/ipc";

/** A structured daemon fault preserved from the wire contract. */
export class DaemonFaultError extends Error {
  readonly fault: Fault;

  constructor(fault: Fault) {
    super(describeFault(fault));
    this.name = "DaemonFaultError";
    this.fault = fault;
  }
}

/** A transport/bridge failure (no local daemon, closed connection, deadline). */
export class BridgeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "BridgeError";
  }
}

const PROFILE_TEXT: Record<ProfileFault, string> = {
  invalid: "The profile data is invalid.",
  unsupported_version: "This profile uses a newer, unsupported schema version.",
  not_found: "The profile was not found.",
  storage: "The local profile store could not be read or written.",
};

export function describeDeviceError(error: DeviceError): string {
  switch (error.code) {
    case "not_supported":
      return "The selected device or driver does not support this operation.";
    case "not_connected":
      return "No live connection is available for this device.";
    case "permission_denied":
      return "The operating system denied access to the device.";
    case "timeout":
      return "The operation timed out. It was not retried automatically.";
    case "incompatible_version":
      return "The SDK contract version is not supported.";
    case "invalid_argument":
      return "The request was rejected as invalid.";
    case "transport_error":
      return "The device transport reported a failure.";
    case "protocol_error":
      return "The device protocol validation failed.";
  }
}

export function describeFault(fault: Fault): string {
  switch (fault.code) {
    case "incompatible_version":
      return "The daemon IPC version is incompatible with this app.";
    case "invalid_argument":
      return "The daemon rejected the request as invalid.";
    case "busy":
      return "The daemon is busy; try again shortly.";
    case "timeout":
      return "The daemon did not answer before the deadline. The operation may have an unknown outcome and must not be replayed.";
    case "internal":
      return "The daemon reported an internal error.";
    case "device":
      return describeDeviceError(fault.detail);
    case "profile":
      return PROFILE_TEXT[fault.detail];
  }
}

export function isUnsupportedFault(fault: Fault): boolean {
  return fault.code === "device" && fault.detail.code === "not_supported";
}

export function isPermissionFault(fault: Fault): boolean {
  return fault.code === "device" && fault.detail.code === "permission_denied";
}
