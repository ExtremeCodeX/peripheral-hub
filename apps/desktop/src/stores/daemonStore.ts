import type { DeviceSummary, Health, MacroSummary, ProfileSummary } from "../types/ipc";

export type ConnectionState = "idle" | "connecting" | "ready" | "error";

export interface DaemonState {
  connection: ConnectionState;
  error: string | null;
  health: Health | null;
  devices: DeviceSummary[];
  devicesError: string | null;
  profiles: ProfileSummary[];
  profilesError: string | null;
  macros: MacroSummary[];
  macrosError: string | null;
}

export const initialDaemonState: DaemonState = {
  connection: "idle",
  error: null,
  health: null,
  devices: [],
  devicesError: null,
  profiles: [],
  profilesError: null,
  macros: [],
  macrosError: null,
};

export type DaemonAction =
  | { type: "connecting" }
  | { type: "connected" }
  | { type: "connectionFailed"; message: string }
  | { type: "health"; health: Health }
  | { type: "devices"; devices: DeviceSummary[] }
  | { type: "devicesError"; message: string }
  | { type: "profiles"; profiles: ProfileSummary[] }
  | { type: "profilesError"; message: string }
  | { type: "macros"; macros: MacroSummary[] }
  | { type: "macrosError"; message: string };

/** Pure reducer; all transitions are covered and exhaustively typed. */
export function daemonReducer(state: DaemonState, action: DaemonAction): DaemonState {
  switch (action.type) {
    case "connecting":
      return { ...state, connection: "connecting", error: null };
    case "connected":
      return { ...state, connection: "ready", error: null };
    case "connectionFailed":
      return { ...state, connection: "error", error: action.message };
    case "health":
      return { ...state, health: action.health };
    case "devices":
      return { ...state, devices: action.devices, devicesError: null };
    case "devicesError":
      return { ...state, devices: [], devicesError: action.message };
    case "profiles":
      return { ...state, profiles: action.profiles, profilesError: null };
    case "profilesError":
      return { ...state, profiles: [], profilesError: action.message };
    case "macros":
      return { ...state, macros: action.macros, macrosError: null };
    case "macrosError":
      return { ...state, macros: [], macrosError: action.message };
  }
}

export interface DaemonStore {
  getState(): DaemonState;
  subscribe(listener: () => void): () => void;
  dispatch(action: DaemonAction): void;
}

/** Minimal observable store, suitable for React's useSyncExternalStore. */
export function createDaemonStore(initial: DaemonState = initialDaemonState): DaemonStore {
  let state = initial;
  const listeners = new Set<() => void>();
  return {
    getState: () => state,
    subscribe(listener: () => void): () => void {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    dispatch(action: DaemonAction): void {
      const next = daemonReducer(state, action);
      if (next === state) {
        return;
      }
      state = next;
      for (const listener of listeners) {
        listener();
      }
    },
  };
}
