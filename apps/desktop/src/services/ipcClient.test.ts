import { describe, expect, it } from "vitest";

import { fakeDaemon } from "../testing/fakeDaemon";
import type { Health } from "../types/ipc";
import { IPC_VERSION } from "../types/ipc";
import type { DaemonTransport } from "./transport";
import { DaemonFaultError } from "./errors";
import { IpcClient } from "./ipcClient";

const health: Health = {
  session_id: "session-1",
  daemon_version: "0.1.0",
  sdk_version: { major: 1, minor: 0 },
  mode: "simulated",
  discovery_error: null,
  sequence: 3,
};

describe("IpcClient", () => {
  it("unwraps a health reply", async () => {
    const { transport } = fakeDaemon(() => ({ result: "health", data: health }));
    const client = new IpcClient(transport);
    await expect(client.health()).resolves.toEqual(health);
  });

  it("maps a fault reply to a typed DaemonFaultError", async () => {
    const { transport } = fakeDaemon(() => ({ code: "profile", detail: "not_found" }));
    const client = new IpcClient(transport);
    await expect(client.getProfile("missing")).rejects.toBeInstanceOf(DaemonFaultError);
    await client.getProfile("missing").catch((error: DaemonFaultError) => {
      expect(error.fault).toEqual({ code: "profile", detail: "not_found" });
    });
  });

  it("rejects an incompatible wire version before typing the payload", async () => {
    const transport: DaemonTransport = {
      async call() {
        return { version: IPC_VERSION + 1, request_id: 1, result: { Ok: { result: "health", data: health } } };
      },
    };
    const client = new IpcClient(transport);
    await expect(client.health()).rejects.toMatchObject({ fault: { code: "incompatible_version" } });
  });

  it("rejects an unexpected reply discriminant", async () => {
    const { transport } = fakeDaemon(() => ({ result: "ack" }));
    const client = new IpcClient(transport);
    await expect(client.listDevices()).rejects.toThrow(/unexpected daemon reply/);
  });

  it("sends typed parameters for set_lighting", async () => {
    const { transport, calls } = fakeDaemon(() => ({ result: "outcome", data: { mode: "simulated" } }));
    const client = new IpcClient(transport);
    await client.setLighting("iface-1", {
      zone: "whole-device",
      color: { red: 1, green: 2, blue: 3 },
      brightness: 50,
      effect: "static",
    });
    expect(calls[0]).toEqual({
      operation: "set_lighting",
      parameters: {
        interface_id: "iface-1",
        command: {
          zone: "whole-device",
          color: { red: 1, green: 2, blue: 3 },
          brightness: 50,
          effect: "static",
        },
      },
    });
  });

  it("sends typed parameters for set_lighting_scene", async () => {
    const { transport, calls } = fakeDaemon(() => ({ result: "outcome", data: { mode: "simulated" } }));
    const client = new IpcClient(transport);
    await expect(
      client.setLightingScene("iface-1", {
        commands: [
          { zone: "whole-device", color: { red: 1, green: 2, blue: 3 }, brightness: 50, effect: "static" },
          { zone: "logo", color: { red: 4, green: 5, blue: 6 }, brightness: null, effect: "static" },
        ],
      }),
    ).resolves.toEqual({ mode: "simulated" });
    expect(calls[0]).toEqual({
      operation: "set_lighting_scene",
      parameters: {
        interface_id: "iface-1",
        scene: {
          commands: [
            { zone: "whole-device", color: { red: 1, green: 2, blue: 3 }, brightness: 50, effect: "static" },
            { zone: "logo", color: { red: 4, green: 5, blue: 6 }, brightness: null, effect: "static" },
          ],
        },
      },
    });
  });

  it("sends typed parameters for apply_profile", async () => {
    const { transport, calls } = fakeDaemon(() => ({ result: "outcome", data: { mode: "simulated" } }));
    const client = new IpcClient(transport);
    await expect(client.applyProfile("iface-1", "gaming")).resolves.toEqual({ mode: "simulated" });
    expect(calls[0]).toEqual({
      operation: "apply_profile",
      parameters: { interface_id: "iface-1", profile_id: "gaming" },
    });
  });

  it("propagates an error reply as a fault", async () => {
    const { transport } = fakeDaemon(() => ({ code: "device", detail: { code: "not_supported" } }));
    const client = new IpcClient(transport);
    await expect(client.connect("iface-1")).rejects.toBeInstanceOf(DaemonFaultError);
  });
});
