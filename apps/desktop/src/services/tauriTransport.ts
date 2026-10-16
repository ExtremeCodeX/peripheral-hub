import { invoke } from "@tauri-apps/api/core";

import type { Operation, Response } from "../types/ipc";
import { BridgeError } from "./errors";
import type { DaemonTransport } from "./transport";

/**
 * Preserve the bridge's stable message. `invoke` rejects with the Rust command
 * error value (a string here), which is not an `Error`; without this wrapper the
 * controller would collapse it into a generic "Unexpected error.".
 */
function describeBridgeFailure(error: unknown): string {
  if (typeof error === "string" && error.length > 0) {
    return error;
  }
  if (error instanceof Error && error.message.length > 0) {
    return error.message;
  }
  return "The desktop bridge rejected the request.";
}

/**
 * Production transport: a single restricted Tauri command.
 *
 * The frontend cannot reach USB/HID or the socket directly. `daemon_call`
 * opens a fresh authorized local IPC connection (same-user daemon) per call,
 * matching the daemon's reconnect model.
 */
export const tauriTransport: DaemonTransport = {
  async call(operation: Operation): Promise<Response> {
    try {
      return (await invoke("daemon_call", { operation })) as Response;
    } catch (error) {
      throw new BridgeError(describeBridgeFailure(error));
    }
  },
};
