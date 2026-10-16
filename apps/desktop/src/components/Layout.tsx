import type { ReactNode } from "react";

import { useDaemon } from "../hooks/useDaemon";
import { ModeBadge } from "./Badge";

export type Screen =
  | "devices"
  | "device"
  | "lighting"
  | "profiles"
  | "macros"
  | "displays"
  | "diagnostics";

const NAV: { id: Screen; label: string }[] = [
  { id: "devices", label: "Devices" },
  { id: "lighting", label: "Lighting" },
  { id: "profiles", label: "Profiles" },
  { id: "macros", label: "Macros" },
  { id: "displays", label: "Displays" },
  { id: "diagnostics", label: "Diagnostics" },
];

const CONNECTION_TEXT: Record<string, string> = {
  idle: "Not connected",
  connecting: "Connecting…",
  ready: "Daemon connected",
  error: "Daemon unavailable",
};

export function Layout({
  screen,
  onNavigate,
  children,
}: {
  screen: Screen;
  onNavigate: (screen: Screen) => void;
  children: ReactNode;
}) {
  const { state } = useDaemon();
  return (
    <div className="app">
      <aside className="sidebar">
        <h1 className="brand">Peripheral Hub</h1>
        <nav aria-label="Primary">
          {NAV.map((item) => (
            <button
              key={item.id}
              type="button"
              className={item.id === screen ? "nav active" : "nav"}
              aria-current={item.id === screen ? "page" : undefined}
              onClick={() => onNavigate(item.id)}
            >
              {item.label}
            </button>
          ))}
        </nav>
      </aside>
      <main className="content">
        <header className="topbar">
          <span className={`conn conn-${state.connection}`}>{CONNECTION_TEXT[state.connection]}</span>
          {state.health ? <ModeBadge mode={state.health.mode} /> : null}
        </header>
        {children}
      </main>
    </div>
  );
}
