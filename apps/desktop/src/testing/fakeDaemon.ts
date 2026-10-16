// In-memory daemon double for tests. It never touches hardware or a socket.

import type { DaemonTransport } from "../services/transport";
import type { Fault, Operation, Reply, Response } from "../types/ipc";
import { IPC_VERSION } from "../types/ipc";

export type Handler = (operation: Operation) => Reply | Fault;

export function ok(reply: Reply, requestId = 1): Response {
  return { version: IPC_VERSION, request_id: requestId, result: { Ok: reply } };
}

export function err(fault: Fault, requestId = 1): Response {
  return { version: IPC_VERSION, request_id: requestId, result: { Err: fault } };
}

export interface FakeDaemon {
  transport: DaemonTransport;
  calls: Operation[];
}

export function fakeDaemon(handler: Handler): FakeDaemon {
  const calls: Operation[] = [];
  let requestId = 0;
  const transport: DaemonTransport = {
    async call(operation: Operation): Promise<Response> {
      calls.push(operation);
      requestId += 1;
      const outcome = handler(operation);
      if ("result" in outcome) {
        return ok(outcome, requestId);
      }
      return err(outcome, requestId);
    },
  };
  return { transport, calls };
}
