import { describe, expect, it } from "vitest";

import type { Capabilities, DeviceSummary } from "../types/ipc";
import {
  capabilityRows,
  connectedCount,
  deviceTitle,
  formatVidPid,
  statusLabel,
  supportLabel,
} from "./viewModel";

function device(overrides: Partial<DeviceSummary> = {}): DeviceSummary {
  return {
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
    ...overrides,
  };
}

describe("viewModel", () => {
  it("formats VID:PID with four hex digits", () => {
    expect(formatVidPid(0x1, 0xabcd)).toBe("0001:abcd");
  });

  it("prefers product then manufacturer then VID:PID", () => {
    expect(deviceTitle(device({ product: "K67" }))).toBe("K67");
    expect(deviceTitle(device({ manufacturer: "ACME" }))).toBe("ACME");
    expect(deviceTitle(device())).toBe("1234:abcd");
  });

  it("labels unknown models as read-only", () => {
    expect(supportLabel(device())).toContain("Unknown");
    expect(supportLabel(device({ model: "simulation:peripheral" }))).toContain(
      "simulation:peripheral",
    );
  });

  it("counts only connected devices", () => {
    expect(
      connectedCount([
        device({ status: "connected" }),
        device({ status: "discovered", interface_id: "iface-2" }),
      ]),
    ).toBe(1);
  });

  it("renders each capability row explicitly", () => {
    const capabilities: Capabilities = {
      lighting: { zones: ["whole-device"], brightness: true, effects: ["static"] },
      display: null,
      buttons: null,
      macros: null,
      profiles: { max_profiles: 4 },
    };
    const rows = capabilityRows(capabilities);
    expect(rows).toHaveLength(5);
    expect(rows.find((row) => row.feature === "Lighting")?.supported).toBe(true);
    expect(rows.find((row) => row.feature === "Display")?.detail).toBe("Not declared");
    expect(rows.find((row) => row.feature === "Profiles")?.detail).toBe("≤4 profiles");
  });

  it("labels statuses", () => {
    expect(statusLabel("failed")).toBe("Failed");
  });
});
