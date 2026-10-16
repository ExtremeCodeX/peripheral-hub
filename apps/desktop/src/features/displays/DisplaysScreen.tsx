import { useEffect, useState } from "react";

import { EmptyNotice, ErrorNotice, UnsupportedNotice } from "../../components/StateNotice";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import { deviceTitle } from "../../services/viewModel";
import type { DisplayCapabilities } from "../../types/ipc";

export function DisplaysScreen() {
  const { state, controller } = useDaemon();
  const connected = state.devices.filter((device) => device.status === "connected");
  const [selected, setSelected] = useState<string | null>(null);
  const [display, setDisplay] = useState<DisplayCapabilities | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);

  const activeId = selected ?? connected[0]?.interface_id ?? null;

  useEffect(() => {
    setDisplay(null);
    setError(null);
    setLoaded(false);
  }, [activeId]);

  async function load(): Promise<void> {
    if (!activeId) {
      return;
    }
    setError(null);
    try {
      const capabilities = await controller.getCapabilities(activeId);
      setDisplay(capabilities.display);
      setLoaded(true);
    } catch (cause) {
      setError(errorMessage(cause));
      setLoaded(true);
    }
  }

  if (connected.length === 0) {
    return (
      <EmptyNotice
        title="No connected device"
        hint="Connect a device with a display to inspect its declared display capabilities."
      />
    );
  }

  return (
    <section>
      <div className="section-head">
        <h2>Displays</h2>
        <button type="button" onClick={() => void load()}>
          Load capabilities
        </button>
      </div>

      <label className="field">
        <span>Device</span>
        <select value={activeId ?? ""} onChange={(event) => setSelected(event.target.value)}>
          {connected.map((device) => (
            <option key={device.interface_id} value={device.interface_id}>
              {deviceTitle(device)}
            </option>
          ))}
        </select>
      </label>

      {error ? <ErrorNotice message={error} onRetry={() => void load()} /> : null}

      {loaded && !error && !display ? <UnsupportedNotice feature="Display" /> : null}

      {display ? (
        <>
          <table className="capabilities">
            <tbody>
              <tr>
                <th scope="row">Resolution</th>
                <td>
                  {display.width}×{display.height}
                </td>
              </tr>
              <tr>
                <th scope="row">Formats</th>
                <td>{display.formats.join(", ")}</td>
              </tr>
              <tr>
                <th scope="row">Max frame rate</th>
                <td>{display.max_frames_per_second} fps</td>
              </tr>
              <tr>
                <th scope="row">Max frame bytes</th>
                <td>{display.max_frame_bytes}</td>
              </tr>
            </tbody>
          </table>
          <div
            className="display-preview"
            style={{ aspectRatio: `${display.width} / ${display.height}` }}
            aria-label="Display preview placeholder"
          >
            <span>Preview only — no frame is transmitted</span>
          </div>
          <p className="notice-hint">
            Sending frames requires a driver with a validated display protocol; it is not implemented.
          </p>
        </>
      ) : null}
    </section>
  );
}
