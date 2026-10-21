// TypeScript mirror of the daemon's typed local IPC contract (IPC_VERSION = 5).
//
// These shapes are intentionally a faithful copy of `crates/ipc/src/protocol.rs`
// and the SDK DTOs it re-uses. The frontend never talks to USB/HID; it only
// builds these typed operations, and the restricted Tauri bridge performs the
// local IPC call. Keep this file in sync when the wire version changes.

export const IPC_VERSION = 5;

export type ExecutionMode = "simulated" | "physical";

export type ConnectionStatus = "discovered" | "connected" | "failed" | "disconnected";

export type LightingEffect = "static" | "breathing" | "color_cycle";

export type PixelFormat = "rgb888" | "rgb565";

export type TransportKind = "hid" | "usb";

export interface RgbColor {
  red: number;
  green: number;
  blue: number;
}

export interface HidUsage {
  page: number;
  usage: number;
}

export interface ContractVersion {
  major: number;
  minor: number;
}

export interface LightingCommand {
  zone: string;
  color: RgbColor;
  brightness: number | null;
  effect: LightingEffect;
}

// A multi-zone lighting scene; the wire enforces at most MAX_SCENE_ZONES unique
// zones and rejects duplicates before any device access.
export interface LightingScene {
  commands: LightingCommand[];
}

export interface LightingCapabilities {
  zones: string[];
  brightness: boolean;
  effects: LightingEffect[];
}

export interface DisplayCapabilities {
  width: number;
  height: number;
  formats: PixelFormat[];
  max_frames_per_second: number;
  max_frame_bytes: number;
}

export interface ButtonCapabilities {
  count: number;
  remapping: boolean;
}

export interface MacroCapabilities {
  max_actions: number;
  max_delay_ms: number;
}

export interface ProfileCapabilities {
  max_profiles: number;
}

export interface Capabilities {
  lighting: LightingCapabilities | null;
  display: DisplayCapabilities | null;
  buttons: ButtonCapabilities | null;
  macros: MacroCapabilities | null;
  profiles: ProfileCapabilities | null;
}

export interface DeviceSummary {
  instance_id: string;
  interface_id: string;
  vendor_id: number;
  product_id: number;
  manufacturer: string | null;
  product: string | null;
  interface_number: number | null;
  usage: HidUsage | null;
  /** Selected model id when a reviewed driver matched, otherwise null (unknown). */
  model: string | null;
  mode: ExecutionMode;
  status: ConnectionStatus;
  last_error: DeviceError | null;
}

export interface Health {
  session_id: string;
  daemon_version: string;
  sdk_version: ContractVersion;
  mode: ExecutionMode;
  discovery_error: DeviceError | null;
  sequence: number;
}

export interface ConnectionEvent {
  sequence: number;
  interface_id: string;
  status: ConnectionStatus;
  mode: ExecutionMode;
}

export interface EventBatch {
  sequence: number;
  gap: boolean;
  events: ConnectionEvent[];
}

export interface DeviceState {
  mode: ExecutionMode;
  lighting: LightingCommand | null;
}

export interface CommandOutcome {
  mode: ExecutionMode;
}

export interface Profile {
  id: string;
  name: string;
  device_model: string | null;
  lighting: LightingCommand | null;
}

export interface ProfileSummary {
  id: string;
  name: string;
  device_model: string | null;
}

export type KeyPress = "down" | "up" | "tap";

// MacroAction is internally tagged: { kind, ... }.
export type MacroAction =
  | { kind: "key"; key: string; press: KeyPress }
  | { kind: "delay"; ms: number };

export interface MacroRepeat {
  times: number;
  interval_ms: number;
}

export interface Macro {
  id: string;
  name: string;
  actions: MacroAction[];
  repeat: MacroRepeat;
  profile: string | null;
}

export interface MacroSummary {
  id: string;
  name: string;
  action_count: number;
  profile: string | null;
}

// DeviceError is adjacently tagged: { code, detail? }.
export type DeviceError =
  | { code: "not_supported" }
  | { code: "not_connected" }
  | { code: "permission_denied" }
  | { code: "timeout" }
  | { code: "incompatible_version" }
  | { code: "invalid_argument"; detail: string }
  | { code: "transport_error"; detail: string }
  | { code: "protocol_error"; detail: string };

export type ProfileFault = "invalid" | "unsupported_version" | "not_found" | "storage";

export type MacroFault = "invalid" | "unsupported_version" | "not_found" | "storage";

// Fault is adjacently tagged: { code, detail? }.
export type Fault =
  | { code: "incompatible_version" }
  | { code: "invalid_argument" }
  | { code: "busy" }
  | { code: "timeout" }
  | { code: "internal" }
  | { code: "device"; detail: DeviceError }
  | { code: "profile"; detail: ProfileFault }
  | { code: "macro"; detail: MacroFault };

// Reply is adjacently tagged: { result, data? }.
export type Reply =
  | { result: "health"; data: Health }
  | { result: "devices"; data: DeviceSummary[] }
  | { result: "device"; data: DeviceSummary }
  | { result: "capabilities"; data: Capabilities }
  | { result: "state"; data: DeviceState }
  | { result: "outcome"; data: CommandOutcome }
  | { result: "events"; data: EventBatch }
  | { result: "profiles"; data: ProfileSummary[] }
  | { result: "profile"; data: Profile }
  | { result: "macros"; data: MacroSummary[] }
  | { result: "macro"; data: Macro }
  | { result: "ack" };

// Operation is adjacently tagged: { operation, parameters? }.
export type Operation =
  | { operation: "health" }
  | { operation: "list_devices" }
  | { operation: "get_device"; parameters: { interface_id: string } }
  | { operation: "get_capabilities"; parameters: { interface_id: string } }
  | { operation: "connect"; parameters: { interface_id: string } }
  | { operation: "disconnect"; parameters: { interface_id: string } }
  | {
      operation: "set_lighting";
      parameters: { interface_id: string; command: LightingCommand };
    }
  | {
      operation: "set_lighting_scene";
      parameters: { interface_id: string; scene: LightingScene };
    }
  | { operation: "read_state"; parameters: { interface_id: string } }
  | { operation: "events"; parameters: { after: number } }
  | { operation: "simulate_presence"; parameters: { present: boolean } }
  | { operation: "list_profiles" }
  | { operation: "get_profile"; parameters: { profile_id: string } }
  | { operation: "save_profile"; parameters: { profile: Profile } }
  | { operation: "delete_profile"; parameters: { profile_id: string } }
  | {
      operation: "apply_profile";
      parameters: { interface_id: string; profile_id: string };
    }
  | { operation: "list_macros" }
  | { operation: "get_macro"; parameters: { macro_id: string } }
  | { operation: "save_macro"; parameters: { macro: Macro } }
  | { operation: "delete_macro"; parameters: { macro_id: string } };

// serde serializes Result<Reply, Fault> as { Ok } | { Err }.
export type ResponseResult = { Ok: Reply } | { Err: Fault };

export interface Response {
  version: number;
  request_id: number;
  result: ResponseResult;
}

/** Narrow a Reply to a specific `result` discriminant without unwrapping. */
export function isReply<K extends Reply["result"]>(
  reply: Reply,
  kind: K,
): reply is Extract<Reply, { result: K }> {
  return reply.result === kind;
}
