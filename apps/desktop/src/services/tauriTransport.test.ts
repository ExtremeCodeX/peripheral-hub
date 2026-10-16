import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { BridgeError } from "./errors";
import { tauriTransport } from "./tauriTransport";
import type { Response } from "../types/ipc";

const response: Response = {
  version: 2,
  request_id: 1,
  result: { Ok: { result: "ack" } },
};

describe("tauriTransport", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("forwards one typed operation to the bridge command", async () => {
    invoke.mockResolvedValueOnce(response);

    await expect(tauriTransport.call({ operation: "health" })).resolves.toBe(response);
    expect(invoke).toHaveBeenCalledWith("daemon_call", { operation: { operation: "health" } });
  });

  it("preserves the bridge message as a BridgeError", async () => {
    invoke.mockRejectedValueOnce("No local daemon is reachable. Start it with `serve`.");

    const error = await tauriTransport.call({ operation: "health" }).catch((reason) => reason);
    expect(error).toBeInstanceOf(BridgeError);
    expect((error as Error).message).toBe("No local daemon is reachable. Start it with `serve`.");
  });

  it("falls back to a stable message for a non-string rejection", async () => {
    invoke.mockRejectedValueOnce(undefined);

    const error = await tauriTransport.call({ operation: "health" }).catch((reason) => reason);
    expect(error).toBeInstanceOf(BridgeError);
    expect((error as Error).message).toBe("The desktop bridge rejected the request.");
  });
});
