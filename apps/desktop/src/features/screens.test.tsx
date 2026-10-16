import type { ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { DaemonProvider } from "../hooks/useDaemon";
import { initialDaemonState, type DaemonState } from "../stores/daemonStore";
import type { DaemonTransport } from "../services/transport";
import type { DeviceSummary, Health } from "../types/ipc";
import { DeviceDetailsScreen } from "./devices/DeviceDetailsScreen";
import { DevicesScreen } from "./devices/DevicesScreen";
import { DiagnosticsScreen } from "./diagnostics/DiagnosticsScreen";
import { DisplaysScreen } from "./displays/DisplaysScreen";
import { LightingScreen } from "./lighting/LightingScreen";
import { MacrosScreen } from "./macros/MacrosScreen";
import { ProfilesScreen } from "./profiles/ProfilesScreen";

const unusedTransport: DaemonTransport = {
  async call() {
    throw new Error("transport must not be called during a static render");
  },
};

function renderScreen(node: ReactNode, state: Partial<DaemonState> = {}): string {
  return renderToStaticMarkup(
    <DaemonProvider
      transport={unusedTransport}
      autoInitialize={false}
      initialState={{ ...initialDaemonState, ...state }}
    >
      {node}
    </DaemonProvider>,
  );
}

const device: DeviceSummary = {
  instance_id: "unit-1",
  interface_id: "iface-1",
  vendor_id: 0x1234,
  product_id: 0xabcd,
  manufacturer: "ACME",
  product: "K67",
  interface_number: 1,
  usage: { page: 1, usage: 6 },
  model: "simulation:peripheral",
  mode: "simulated",
  status: "discovered",
  last_error: null,
};

const health: Health = {
  session_id: "session-1",
  daemon_version: "0.1.0",
  sdk_version: { major: 1, minor: 0 },
  mode: "simulated",
  discovery_error: null,
  sequence: 4,
};

describe("DevicesScreen", () => {
  it("shows a loading state while connecting", () => {
    expect(renderScreen(<DevicesScreen onOpenDevice={() => {}} />, { connection: "connecting" })).toContain(
      "Discovering devices",
    );
  });

  it("shows an empty state with no devices", () => {
    expect(renderScreen(<DevicesScreen onOpenDevice={() => {}} />, { connection: "ready" })).toContain(
      "No devices discovered",
    );
  });

  it("renders a device with its model and status", () => {
    const html = renderScreen(<DevicesScreen onOpenDevice={() => {}} />, {
      connection: "ready",
      health,
      devices: [device],
    });
    expect(html).toContain("K67");
    expect(html).toContain("Discovered");
    expect(html).toContain("simulation:peripheral");
    expect(html).toContain("Simulate present");
  });

  it("surfaces a discovery error instead of an empty success", () => {
    const html = renderScreen(<DevicesScreen onOpenDevice={() => {}} />, {
      connection: "ready",
      devicesError: "The daemon did not answer.",
    });
    expect(html).toContain("The daemon did not answer.");
    expect(html).not.toContain("No devices discovered");
  });
});

describe("DeviceDetailsScreen", () => {
  it("asks the user to connect before loading capabilities", () => {
    const html = renderScreen(
      <DeviceDetailsScreen interfaceId="iface-1" onOpenLighting={() => {}} onBack={() => {}} />,
      { connection: "ready", devices: [device] },
    );
    expect(html).toContain("iface-1");
    expect(html).toContain("1234:abcd");
    expect(html).toContain("Connect this device to load its declared capabilities.");
  });
});

describe("Lighting and Displays screens", () => {
  it("reports a missing selection", () => {
    expect(renderScreen(<LightingScreen interfaceId={null} />, { connection: "ready" })).toContain(
      "No device selected",
    );
  });

  it("blocks lighting until the device is connected", () => {
    expect(
      renderScreen(<LightingScreen interfaceId="iface-1" />, { connection: "ready", devices: [device] }),
    ).toContain("Device not connected");
  });

  it("shows no connected device on the displays screen", () => {
    expect(renderScreen(<DisplaysScreen />, { connection: "ready", devices: [device] })).toContain(
      "No connected device",
    );
  });
});

describe("ProfilesScreen", () => {
  it("shows an empty state and the create form", () => {
    const html = renderScreen(<ProfilesScreen />, { connection: "ready" });
    expect(html).toContain("No profiles yet");
    expect(html).toContain("Save profile");
  });

  it("lists profiles with their ids", () => {
    const html = renderScreen(<ProfilesScreen />, {
      connection: "ready",
      profiles: [{ id: "gaming", name: "Gaming", device_model: "simulation:peripheral" }],
    });
    expect(html).toContain("Gaming");
    expect(html).toContain("gaming");
  });
});

describe("DiagnosticsScreen", () => {
  it("shows the daemon contract and simulated mode", () => {
    const html = renderScreen(<DiagnosticsScreen />, { connection: "ready", health });
    expect(html).toContain("0.1.0");
    expect(html).toContain("Simulated");
    expect(html).toContain("session-1");
    expect(html).toContain("1.0");
  });

  it("states that no health is available yet", () => {
    expect(renderScreen(<DiagnosticsScreen />, { connection: "ready" })).toContain(
      "No health information yet",
    );
  });
});

describe("MacrosScreen", () => {
  it("renders a standalone simulator without implying hardware control", () => {
    const html = renderToStaticMarkup(<MacrosScreen />);
    expect(html).toContain("Editing, validation and safe simulation only.");
    expect(html).toContain("does not inject keystrokes");
  });
});
