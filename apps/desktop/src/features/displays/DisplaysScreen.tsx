import { useEffect, useState } from "react";

import { EmptyNotice, ErrorNotice, UnsupportedNotice } from "../../components/StateNotice";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import { deviceTitle } from "../../services/viewModel";
import type { DisplayCapabilities, PixelFormat } from "../../types/ipc";
import {
  DISPLAY_PATTERNS,
  type DisplayPattern,
  type FrameSelection,
  defaultSelection,
  previewFrame,
  validateFrame,
} from "./displayModel";

const FORMAT_LABEL: Record<PixelFormat, string> = {
  rgb888: "RGB888 (3 bytes/pixel)",
  rgb565: "RGB565 (2 bytes/pixel)",
};

function clampByte(value: string): number {
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed)) {
    return 0;
  }
  return Math.min(255, Math.max(0, parsed));
}

export function DisplaysScreen() {
  const { state, controller } = useDaemon();
  const connected = state.devices.filter((device) => device.status === "connected");
  const [selected, setSelected] = useState<string | null>(null);
  const [display, setDisplay] = useState<DisplayCapabilities | null>(null);
  const [selection, setSelection] = useState<FrameSelection | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);

  const activeId = selected ?? connected[0]?.interface_id ?? null;

  useEffect(() => {
    setDisplay(null);
    setSelection(null);
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
      setSelection(capabilities.display ? defaultSelection(capabilities.display) : null);
      setLoaded(true);
    } catch (cause) {
      setError(errorMessage(cause));
      setLoaded(true);
    }
  }

  function setColor(channel: "red" | "green" | "blue", value: string): void {
    if (!selection) {
      return;
    }
    setSelection({
      ...selection,
      color: { ...selection.color, [channel]: clampByte(value) },
    });
  }

  if (connected.length === 0) {
    return (
      <EmptyNotice
        title="No connected device"
        hint="Connect a device with a display to inspect its declared display capabilities."
      />
    );
  }

  const problems = display && selection ? validateFrame(display, selection.format) : [];
  const preview = display && selection ? previewFrame(display, selection) : null;

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

      {display && selection && preview ? (
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
                <td>{display.formats.map((format) => FORMAT_LABEL[format]).join(", ")}</td>
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

          <h3>Local preview</h3>
          <div className="form">
            <label className="field">
              <span>Format</span>
              <select
                value={selection.format}
                onChange={(event) =>
                  setSelection({ ...selection, format: event.target.value as PixelFormat })
                }
              >
                {display.formats.map((format) => (
                  <option key={format} value={format}>
                    {FORMAT_LABEL[format]}
                  </option>
                ))}
              </select>
            </label>
            <label className="field">
              <span>Pattern</span>
              <select
                value={selection.pattern}
                onChange={(event) =>
                  setSelection({ ...selection, pattern: event.target.value as DisplayPattern })
                }
              >
                {DISPLAY_PATTERNS.map((pattern) => (
                  <option key={pattern} value={pattern}>
                    {pattern}
                  </option>
                ))}
              </select>
            </label>
            <label className="field">
              <span>Red</span>
              <input
                type="number"
                min={0}
                max={255}
                value={selection.color.red}
                onChange={(event) => setColor("red", event.target.value)}
              />
            </label>
            <label className="field">
              <span>Green</span>
              <input
                type="number"
                min={0}
                max={255}
                value={selection.color.green}
                onChange={(event) => setColor("green", event.target.value)}
              />
            </label>
            <label className="field">
              <span>Blue</span>
              <input
                type="number"
                min={0}
                max={255}
                value={selection.color.blue}
                onChange={(event) => setColor("blue", event.target.value)}
              />
            </label>
          </div>

          {problems.length > 0 ? (
            <div className="notice notice-error" role="alert">
              <p className="notice-title">Frame does not fit the declared contract</p>
              <ul>
                {problems.map((problem) => (
                  <li key={problem}>{problem}</li>
                ))}
              </ul>
            </div>
          ) : null}

          <div
            className="display-preview"
            style={{
              aspectRatio: `${display.width} / ${display.height}`,
              background: preview.background,
            }}
            role="img"
            aria-label={preview.label}
          >
            <span className="display-preview-label">
              {display.width}×{display.height} · {preview.format} · {preview.bytes} bytes
            </span>
          </div>

          <p className="notice-hint">
            This preview is rendered locally in this window. No frame is encoded or transmitted;
            effective display control requires a driver with a validated display protocol, which is
            not implemented.
          </p>
        </>
      ) : null}
    </section>
  );
}
