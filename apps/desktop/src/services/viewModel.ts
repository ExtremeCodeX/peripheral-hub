import type {
  Capabilities,
  ConnectionStatus,
  DeviceSummary,
  ExecutionMode,
  ProfileSummary,
} from "../types/ipc";

export function modeLabel(mode: ExecutionMode): string {
  return mode === "simulated" ? "Simulated" : "Physical";
}

export function statusLabel(status: ConnectionStatus): string {
  switch (status) {
    case "discovered":
      return "Discovered";
    case "connected":
      return "Connected";
    case "failed":
      return "Failed";
    case "disconnected":
      return "Disconnected";
  }
}

export function hex4(value: number): string {
  return value.toString(16).padStart(4, "0");
}

export function formatVidPid(vendorId: number, productId: number): string {
  return `${hex4(vendorId)}:${hex4(productId)}`;
}

/** Human title: prefer the observed product, then manufacturer, then VID:PID. */
export function deviceTitle(device: DeviceSummary): string {
  const product = device.product?.trim();
  if (product) {
    return product;
  }
  const manufacturer = device.manufacturer?.trim();
  if (manufacturer) {
    return manufacturer;
  }
  return formatVidPid(device.vendor_id, device.product_id);
}

export function deviceSubtitle(device: DeviceSummary): string {
  const parts = [formatVidPid(device.vendor_id, device.product_id)];
  if (device.manufacturer && device.product) {
    parts.unshift(device.manufacturer);
  }
  if (device.interface_number !== null) {
    parts.push(`iface ${device.interface_number}`);
  }
  return parts.join(" · ");
}

export type SupportLevel = "supported" | "discovered-unknown";

/**
 * A matched model means a reviewed driver can be offered, not that any physical
 * capability was validated on this unit. Unknown devices are never opened.
 */
export function supportLevel(device: DeviceSummary): SupportLevel {
  return device.model ? "supported" : "discovered-unknown";
}

export function supportLabel(device: DeviceSummary): string {
  return device.model
    ? `Reviewed driver: ${device.model}`
    : "Unknown model — read-only, not opened";
}

export function connectedCount(devices: DeviceSummary[]): number {
  return devices.filter((device) => device.status === "connected").length;
}

export interface CapabilityRow {
  feature: string;
  supported: boolean;
  detail: string;
}

/** Present each capability explicitly, including "not declared" ones. */
export function capabilityRows(capabilities: Capabilities): CapabilityRow[] {
  const rows: CapabilityRow[] = [];

  const lighting = capabilities.lighting;
  rows.push({
    feature: "Lighting",
    supported: Boolean(lighting),
    detail: lighting
      ? `${lighting.zones.length} zone(s), ${lighting.effects.join(", ")}${
          lighting.brightness ? ", brightness" : ""
        }`
      : "Not declared",
  });

  const display = capabilities.display;
  rows.push({
    feature: "Display",
    supported: Boolean(display),
    detail: display
      ? `${display.width}x${display.height}, ${display.formats.join("/")}, ≤${display.max_frames_per_second} fps`
      : "Not declared",
  });

  const buttons = capabilities.buttons;
  rows.push({
    feature: "Buttons",
    supported: Boolean(buttons),
    detail: buttons
      ? `${buttons.count} button(s)${buttons.remapping ? ", remapping" : ", no remapping"}`
      : "Not declared",
  });

  const macros = capabilities.macros;
  rows.push({
    feature: "Macros",
    supported: Boolean(macros),
    detail: macros
      ? `≤${macros.max_actions} actions, ≤${macros.max_delay_ms} ms delay`
      : "Not declared",
  });

  const profiles = capabilities.profiles;
  rows.push({
    feature: "Profiles",
    supported: Boolean(profiles),
    detail: profiles ? `≤${profiles.max_profiles} profiles` : "Not declared",
  });

  return rows;
}

export function profileLabel(profile: ProfileSummary): string {
  return profile.device_model ? `${profile.name} (${profile.device_model})` : profile.name;
}
