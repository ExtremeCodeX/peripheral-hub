import { describe, expect, it } from "vitest";

import { fakeDaemon } from "../testing/fakeDaemon";
import type { DaemonStore } from "../stores/daemonStore";
import { createDaemonStore } from "../stores/daemonStore";
import type { DeviceSummary, Fault, Health, Operation, ProfileSummary, Reply } from "../types/ipc";
import { DaemonController, errorMessage } from "./daemonController";
import { DaemonFaultError } from "./errors";
import { IpcClient } from "./ipcClient";

const health: Health = {
  session_id: "session-1",
  daemon_version: "0.1.0",
  sdk_version: { major: 1, minor: 0 },
  mode: "simulated",
  discovery_error: null,
  sequence: 1,
};

const device: DeviceSummary = {
  instance_id: "unit-1",
  interface_id: "iface-1",
  vendor_id: 0x1234,
  product_id: 0xabcd,
  manufacturer: null,
  product: "K67",
  interface_number: null,
  usage: null,
  model: "simulation:peripheral",
  mode: "simulated",
  status: "discovered",
  last_error: null,
};

const profile: ProfileSummary = { id: "gaming", name: "Gaming", device_model: null };

function connectedStore(): DaemonStore {
  const store = createDaemonStore();
  store.dispatch({ type: "connected" });
  return store;
}

describe("DaemonController", () => {
  it("initializes from health, devices and profiles in order", async () => {
    const seen: string[] = [];
    const { transport } = fakeDaemon((operation: Operation): Reply | Fault => {
      seen.push(operation.operation);
      switch (operation.operation) {
        case "health":
          return { result: "health", data: health };
        case "list_devices":
          return { result: "devices", data: [device] };
        case "list_profiles":
          return { result: "profiles", data: [profile] };
        default:
          return { code: "internal" };
      }
    });
    const store = createDaemonStore();
    await new DaemonController(new IpcClient(transport), store).initialize();

    expect(seen).toEqual(["health", "list_devices", "list_profiles"]);
    expect(store.getState().connection).toBe("ready");
    expect(store.getState().health).toEqual(health);
    expect(store.getState().devices).toHaveLength(1);
    expect(store.getState().profiles).toHaveLength(1);
  });

  it("records a connection failure without listing", async () => {
    const seen: string[] = [];
    const { transport } = fakeDaemon((operation: Operation): Reply | Fault => {
      seen.push(operation.operation);
      return { code: "incompatible_version" };
    });
    const store = createDaemonStore();
    await new DaemonController(new IpcClient(transport), store).initialize();

    expect(seen).toEqual(["health"]);
    expect(store.getState().connection).toBe("error");
    expect(store.getState().error).toContain("incompatible");
  });

  it("refreshes the device list after connecting", async () => {
    let listCalls = 0;
    const { transport } = fakeDaemon((operation: Operation): Reply | Fault => {
      switch (operation.operation) {
        case "connect":
          return { result: "device", data: { ...device, status: "connected" } };
        case "list_devices":
          listCalls += 1;
          return { result: "devices", data: [{ ...device, status: "connected" }] };
        default:
          return { code: "internal" };
      }
    });
    const store = connectedStore();
    const returned = await new DaemonController(new IpcClient(transport), store).connectDevice("iface-1");

    expect(returned.status).toBe("connected");
    expect(listCalls).toBe(1);
    expect(store.getState().devices[0]?.status).toBe("connected");
  });

  it("surfaces a device fault to the caller and keeps prior state", async () => {
    const { transport } = fakeDaemon(() => ({ code: "device", detail: { code: "not_connected" } }));
    const store = connectedStore();
    const controller = new DaemonController(new IpcClient(transport), store);
    await expect(controller.connectDevice("iface-1")).rejects.toBeInstanceOf(DaemonFaultError);
  });

  it("stores a devices error instead of an empty success", async () => {
    const { transport } = fakeDaemon(() => ({ code: "timeout" }));
    const store = connectedStore();
    await new DaemonController(new IpcClient(transport), store).refreshDevices();
    expect(store.getState().devices).toEqual([]);
    expect(store.getState().devicesError).toMatch(/deadline|unknown outcome/);
  });

  it("refreshes profiles after saving", async () => {
    const { transport } = fakeDaemon((operation: Operation): Reply | Fault => {
      switch (operation.operation) {
        case "save_profile":
          return { result: "profile", data: { ...operation.parameters.profile } };
        case "list_profiles":
          return { result: "profiles", data: [profile] };
        default:
          return { code: "internal" };
      }
    });
    const store = connectedStore();
    const saved = await new DaemonController(new IpcClient(transport), store).saveProfile({
      id: "gaming",
      name: "Gaming",
      device_model: null,
      lighting: null,
    });
    expect(saved.id).toBe("gaming");
    expect(store.getState().profiles).toEqual([profile]);
  });

  it("describes faults and unknown errors", () => {
    expect(errorMessage(new DaemonFaultError({ code: "busy" }))).toContain("busy");
    expect(errorMessage(new Error("boom"))).toBe("boom");
    expect(errorMessage("nope")).toBe("Unexpected error.");
  });
});
