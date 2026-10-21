import { useEffect, useState } from "react";

import { ModeBadge } from "../../components/Badge";
import { EmptyNotice, ErrorNotice, LoadingNotice, UnsupportedNotice } from "../../components/StateNotice";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import { deviceTitle } from "../../services/viewModel";
import type { Capabilities, LightingEffect } from "../../types/ipc";

function clampByte(value: string): number {
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed)) {
    return 0;
  }
  return Math.min(255, Math.max(0, parsed));
}

export function LightingScreen({ interfaceId }: { interfaceId: string | null }) {
  const { state, controller } = useDaemon();
  const device = interfaceId
    ? state.devices.find((candidate) => candidate.interface_id === interfaceId)
    : undefined;

  const [capabilities, setCapabilities] = useState<Capabilities | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [selectedZones, setSelectedZones] = useState<string[]>([]);
  const [red, setRed] = useState("255");
  const [green, setGreen] = useState("0");
  const [blue, setBlue] = useState("0");
  const [brightness, setBrightness] = useState("100");
  const [effect, setEffect] = useState<LightingEffect>("static");

  const connected = device?.status === "connected";

  useEffect(() => {
    setCapabilities(null);
    setError(null);
    setSuccess(null);
  }, [interfaceId]);

  useEffect(() => {
    if (device && connected) {
      void load();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [interfaceId, connected]);

  async function load(): Promise<void> {
    if (!interfaceId) {
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const result = await controller.getCapabilities(interfaceId);
      setCapabilities(result);
      setSelectedZones(result.lighting ? result.lighting.zones.slice(0, 1) : []);
      if (result.lighting && result.lighting.effects.length > 0) {
        setEffect(result.lighting.effects[0] as LightingEffect);
      }
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setLoading(false);
    }
  }

  function toggleZone(zone: string): void {
    setSelectedZones((current) =>
      current.includes(zone) ? current.filter((entry) => entry !== zone) : [...current, zone],
    );
  }

  async function apply(): Promise<void> {
    if (!interfaceId) {
      return;
    }
    setError(null);
    setSuccess(null);
    const lighting = capabilities?.lighting;
    if (!lighting) {
      return;
    }
    if (selectedZones.length === 0) {
      setError("Select at least one zone to apply the scene.");
      return;
    }
    try {
      const outcome = await controller.setLightingScene(interfaceId, {
        commands: selectedZones.map((zone) => ({
          zone,
          color: { red: clampByte(red), green: clampByte(green), blue: clampByte(blue) },
          brightness: lighting.brightness ? clampByte(brightness) : null,
          effect,
        })),
      });
      const zones = `${selectedZones.length} zone${selectedZones.length === 1 ? "" : "s"}`;
      setSuccess(
        outcome.mode === "simulated"
          ? `Applied ${zones} in simulation (no physical device was written).`
          : `Lighting scene for ${zones} reported as successful by the driver.`,
      );
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  if (!interfaceId || !device) {
    return <EmptyNotice title="No device selected" hint="Choose a device on the Devices screen." />;
  }
  if (!connected) {
    return <EmptyNotice title="Device not connected" hint="Connect the device first to control lighting." />;
  }
  if (loading && !capabilities) {
    return <LoadingNotice label="Loading lighting capabilities…" />;
  }
  if (error && !capabilities) {
    return <ErrorNotice message={error} onRetry={() => void load()} />;
  }
  if (capabilities && !capabilities.lighting) {
    return <UnsupportedNotice feature="Lighting" />;
  }

  const lighting = capabilities?.lighting;

  return (
    <section>
      <div className="section-head">
        <h2>Lighting — {deviceTitle(device)}</h2>
        <ModeBadge mode={device.mode} />
      </div>

      {device.mode === "simulated" ? (
        <p className="notice-hint">Simulation mode: changes are not sent to physical hardware.</p>
      ) : null}
      {error ? <ErrorNotice message={error} /> : null}
      {success ? (
        <p className="notice" role="status">
          {success}
        </p>
      ) : null}

      {lighting ? (
        <form
          className="form"
          onSubmit={(event) => {
            event.preventDefault();
            void apply();
          }}
        >
          <fieldset className="zones">
            <legend>Zones</legend>
            {lighting.zones.map((candidate) => (
              <label key={candidate} className="field checkbox">
                <input
                  type="checkbox"
                  checked={selectedZones.includes(candidate)}
                  onChange={() => toggleZone(candidate)}
                />
                <span>{candidate}</span>
              </label>
            ))}
            <button
              type="button"
              className="link"
              onClick={() =>
                setSelectedZones(
                  selectedZones.length === lighting.zones.length ? [] : [...lighting.zones],
                )
              }
            >
              {selectedZones.length === lighting.zones.length ? "Clear all" : "Select all"}
            </button>
          </fieldset>

          <fieldset className="rgb">
            <legend>Color</legend>
            <label className="field">
              <span>Red</span>
              <input type="number" min={0} max={255} value={red} onChange={(event) => setRed(event.target.value)} />
            </label>
            <label className="field">
              <span>Green</span>
              <input type="number" min={0} max={255} value={green} onChange={(event) => setGreen(event.target.value)} />
            </label>
            <label className="field">
              <span>Blue</span>
              <input type="number" min={0} max={255} value={blue} onChange={(event) => setBlue(event.target.value)} />
            </label>
          </fieldset>

          {lighting.brightness ? (
            <label className="field">
              <span>Brightness (0–100)</span>
              <input
                type="number"
                min={0}
                max={100}
                value={brightness}
                onChange={(event) => setBrightness(event.target.value)}
              />
            </label>
          ) : null}

          <label className="field">
            <span>Effect</span>
            <select value={effect} onChange={(event) => setEffect(event.target.value as LightingEffect)}>
              {lighting.effects.map((candidate) => (
                <option key={candidate} value={candidate}>
                  {candidate}
                </option>
              ))}
            </select>
          </label>

          <button type="submit" disabled={selectedZones.length === 0}>
            Apply to {selectedZones.length} zone{selectedZones.length === 1 ? "" : "s"}
          </button>
        </form>
      ) : null}
    </section>
  );
}
