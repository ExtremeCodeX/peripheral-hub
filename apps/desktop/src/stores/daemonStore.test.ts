import { describe, expect, it } from "vitest";

import type { DeviceSummary } from "../types/ipc";
import { createDaemonStore, daemonReducer, initialDaemonState } from "./daemonStore";

const device: DeviceSummary = {
  instance_id: "unit-1",
  interface_id: "iface-1",
  vendor_id: 0x1234,
  product_id: 0xabcd,
  manufacturer: null,
  product: null,
  interface_number: null,
  usage: null,
  model: null,
  mode: "simulated",
  status: "discovered",
  last_error: null,
};

describe("daemonReducer", () => {
  it("moves idle -> connecting -> ready", () => {
    const connecting = daemonReducer(initialDaemonState, { type: "connecting" });
    expect(connecting.connection).toBe("connecting");
    expect(daemonReducer(connecting, { type: "connected" }).connection).toBe("ready");
  });

  it("records a connection failure message", () => {
    const next = daemonReducer(initialDaemonState, { type: "connectionFailed", message: "no daemon" });
    expect(next.connection).toBe("error");
    expect(next.error).toBe("no daemon");
  });

  it("clears the devices error when devices arrive", () => {
    const failed = daemonReducer(initialDaemonState, { type: "devicesError", message: "boom" });
    expect(failed.devicesError).toBe("boom");
    const loaded = daemonReducer(failed, { type: "devices", devices: [device] });
    expect(loaded.devicesError).toBeNull();
    expect(loaded.devices).toHaveLength(1);
  });

  it("clears the macros error when macros arrive", () => {
    const failed = daemonReducer(initialDaemonState, { type: "macrosError", message: "boom" });
    expect(failed.macrosError).toBe("boom");
    const loaded = daemonReducer(failed, {
      type: "macros",
      macros: [{ id: "combo", name: "Combo", action_count: 2, profile: null }],
    });
    expect(loaded.macrosError).toBeNull();
    expect(loaded.macros).toHaveLength(1);
  });

  it("notifies subscribers on real changes only", () => {
    const store = createDaemonStore();
    let notifications = 0;
    const unsubscribe = store.subscribe(() => {
      notifications += 1;
    });
    store.dispatch({ type: "connecting" });
    store.dispatch({ type: "devices", devices: [device] });
    unsubscribe();
    store.dispatch({ type: "connected" });
    expect(notifications).toBe(2);
    expect(store.getState().connection).toBe("ready");
    expect(store.getState().devices).toHaveLength(1);
  });
});
