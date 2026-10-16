import { createContext, useContext, useEffect, useMemo, useRef, useSyncExternalStore } from "react";
import type { ReactNode } from "react";

import { DaemonController } from "../services/daemonController";
import { IpcClient } from "../services/ipcClient";
import { tauriTransport } from "../services/tauriTransport";
import type { DaemonTransport } from "../services/transport";
import { createDaemonStore, type DaemonState, type DaemonStore } from "../stores/daemonStore";

interface DaemonContextValue {
  state: DaemonState;
  controller: DaemonController;
}

const DaemonContext = createContext<DaemonContextValue | null>(null);

export interface DaemonProviderProps {
  children: ReactNode;
  /** Override the transport in tests; production uses the Tauri bridge. */
  transport?: DaemonTransport;
  autoInitialize?: boolean;
  /** Seed state for tests and server rendering. */
  initialState?: DaemonState;
}

export function DaemonProvider({
  children,
  transport = tauriTransport,
  autoInitialize = true,
  initialState,
}: DaemonProviderProps) {
  const storeRef = useRef<DaemonStore | null>(null);
  if (storeRef.current === null) {
    storeRef.current = createDaemonStore(initialState);
  }
  const store = storeRef.current;

  const controller = useMemo(() => new DaemonController(new IpcClient(transport), store), [transport, store]);

  // The third argument is the server snapshot; without it `useSyncExternalStore`
  // throws during server rendering, which the render-smoke tests rely on.
  const state = useSyncExternalStore(store.subscribe, store.getState, store.getState);

  useEffect(() => {
    if (autoInitialize) {
      void controller.initialize();
    }
  }, [controller, autoInitialize]);

  const value = useMemo<DaemonContextValue>(() => ({ state, controller }), [state, controller]);

  return <DaemonContext.Provider value={value}>{children}</DaemonContext.Provider>;
}

export function useDaemon(): DaemonContextValue {
  const value = useContext(DaemonContext);
  if (value === null) {
    throw new Error("useDaemon must be used within a DaemonProvider");
  }
  return value;
}
