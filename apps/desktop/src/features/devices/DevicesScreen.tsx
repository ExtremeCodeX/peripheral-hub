import { useState } from "react";

import { ModeBadge, StatusBadge } from "../../components/Badge";
import { EmptyNotice, ErrorNotice, LoadingNotice } from "../../components/StateNotice";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import { deviceSubtitle, deviceTitle, supportLabel } from "../../services/viewModel";

export function DevicesScreen({ onOpenDevice }: { onOpenDevice: (interfaceId: string) => void }) {
  const { state, controller } = useDaemon();
  const [busyId, setBusyId] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  if (state.connection === "idle" || state.connection === "connecting") {
    return <LoadingNotice label="Discovering devices…" />;
  }
  if (state.connection === "error") {
    return (
      <ErrorNotice
        message={state.error ?? "The daemon is unavailable."}
        onRetry={() => void controller.initialize()}
      />
    );
  }

  async function runAction(id: string, action: () => Promise<unknown>): Promise<void> {
    setBusyId(id);
    setActionError(null);
    try {
      await action();
    } catch (error) {
      setActionError(errorMessage(error));
    } finally {
      setBusyId(null);
    }
  }

  const simulated = state.health?.mode === "simulated";

  return (
    <section>
      <div className="section-head">
        <h2>Devices</h2>
        <button type="button" onClick={() => void controller.refreshDevices()}>
          Refresh
        </button>
      </div>

      {state.devicesError ? (
        <ErrorNotice message={state.devicesError} onRetry={() => void controller.refreshDevices()} />
      ) : null}
      {actionError ? <ErrorNotice message={actionError} /> : null}

      {state.devices.length === 0 && !state.devicesError ? (
        <EmptyNotice
          title="No devices discovered"
          hint="Connect a device, or start the daemon in simulation mode (serve --simulate)."
        />
      ) : null}

      <ul className="device-list">
        {state.devices.map((device) => (
          <li key={device.interface_id} className="device-card">
            <div className="device-main">
              <button
                type="button"
                className="device-title"
                onClick={() => onOpenDevice(device.interface_id)}
              >
                {deviceTitle(device)}
              </button>
              <span className="device-sub">{deviceSubtitle(device)}</span>
              <span className="device-support">{supportLabel(device)}</span>
            </div>
            <div className="device-meta">
              <StatusBadge status={device.status} />
              <ModeBadge mode={device.mode} />
            </div>
            <div className="device-actions">
              {device.status === "connected" ? (
                <button
                  type="button"
                  disabled={busyId === device.interface_id}
                  onClick={() =>
                    void runAction(device.interface_id, () =>
                      controller.disconnectDevice(device.interface_id),
                    )
                  }
                >
                  Disconnect
                </button>
              ) : (
                <button
                  type="button"
                  disabled={busyId === device.interface_id || device.model === null}
                  title={device.model === null ? "No reviewed driver matched this device" : undefined}
                  onClick={() =>
                    void runAction(device.interface_id, () =>
                      controller.connectDevice(device.interface_id),
                    )
                  }
                >
                  Connect
                </button>
              )}
              <button type="button" onClick={() => onOpenDevice(device.interface_id)}>
                Details
              </button>
            </div>
          </li>
        ))}
      </ul>

      {simulated ? (
        <div className="sim-controls">
          <p className="notice-hint">Simulation controls (software only, no hardware I/O):</p>
          <button type="button" onClick={() => void controller.setSimulatedPresence(true)}>
            Simulate present
          </button>
          <button type="button" onClick={() => void controller.setSimulatedPresence(false)}>
            Simulate absent
          </button>
        </div>
      ) : null}
    </section>
  );
}
