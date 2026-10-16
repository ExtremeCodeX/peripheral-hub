import type { Operation, Response } from "../types/ipc";

/**
 * A single typed request/response channel to the daemon.
 *
 * The real implementation is the restricted Tauri bridge (`@tauri-apps/api`).
 * Tests inject an in-memory fake, so no device or socket is required.
 */
export interface DaemonTransport {
  call(operation: Operation): Promise<Response>;
}
