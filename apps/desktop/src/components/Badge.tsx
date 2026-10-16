import type { ReactNode } from "react";

import { statusLabel } from "../services/viewModel";
import type { ConnectionStatus, ExecutionMode } from "../types/ipc";

export type Tone = "ok" | "warn" | "muted" | "error";

export function Badge({ tone, children }: { tone: Tone; children: ReactNode }) {
  return <span className={`badge badge-${tone}`}>{children}</span>;
}

export function StatusBadge({ status }: { status: ConnectionStatus }) {
  const tone: Tone =
    status === "connected"
      ? "ok"
      : status === "failed"
        ? "error"
        : status === "discovered"
          ? "warn"
          : "muted";
  return <Badge tone={tone}>{statusLabel(status)}</Badge>;
}

export function ModeBadge({ mode }: { mode: ExecutionMode }) {
  return <Badge tone={mode === "simulated" ? "warn" : "muted"}>{mode === "simulated" ? "Simulated" : "Physical"}</Badge>;
}
