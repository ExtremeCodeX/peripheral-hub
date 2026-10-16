import { useEffect, useState } from "react";

import { ModeBadge, StatusBadge } from "../../components/Badge";
import { EmptyNotice, ErrorNotice, LoadingNotice } from "../../components/StateNotice";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import { capabilityRows, deviceSubtitle, deviceTitle, formatVidPid, supportLabel } from "../../services/viewModel";
import type { Capabilities } from "../../types/ipc";

export function DeviceDetailsScreen({
  interfaceId,
  onOpenLighting,
  onBack,
}: {
  interfaceId: string | null;
  onOpenLighting: (interfaceId: string) => void;
  onBack: () => void;
}) {
  const { state, controller } = useDaemon();
  const device = interfaceId
    ? state.devices.find((candidate) => candidate.interface_id === interfaceId)
    : undefined;

  const [capabilities, setCapabilities] = useState<Capabilities | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setCapabilities(null);
    setError(null);
  }, [interfaceId]);

  if (!interfaceId) {
    return <EmptyNotice title="No device selected" hint="Choose a device from the Devices screen." />;
  }
  if (!device) {
    return <EmptyNotice title="Device not found" hint="It may have been removed or the list changed." />;
  }

  async function loadCapabilities(): Promise<void> {
    setLoading(true);
    setError(null);
    try {
      setCapabilities(await controller.getCapabilities(interfaceId as string));
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setLoading(false);
    }
  }

  const rows = capabilities ? capabilityRows(capabilities) : [];

  return (
    <section>
      <div className="section-head">
        <h2>{deviceTitle(device)}</h2>
        <button type="button" onClick={onBack}>
          Back
        </button>
      </div>

      <div className="detail-meta">
        <StatusBadge status={device.status} />
        <ModeBadge mode={device.mode} />
      </div>

      <dl className="facts">
        <dt>Endpoint</dt>
        <dd>{device.interface_id}</dd>
        <dt>USB id</dt>
        <dd>{formatVidPid(device.vendor_id, device.product_id)}</dd>
        <dt>Transport summary</dt>
        <dd>{deviceSubtitle(device)}</dd>
        <dt>Driver</dt>
        <dd>{supportLabel(device)}</dd>
        <dt>Usage</dt>
        <dd>{device.usage ? `page ${device.usage.page}, usage ${device.usage.usage}` : "unknown"}</dd>
      </dl>

      <p className="notice-hint">
        A matched driver means a reviewed implementation may be available; it does not prove that a
        capability works on this exact unit.
      </p>

      <div className="section-head">
        <h3>Capabilities</h3>
        {device.status === "connected" ? (
          <button type="button" disabled={loading} onClick={() => void loadCapabilities()}>
            {loading ? "Loading…" : "Load capabilities"}
          </button>
        ) : null}
      </div>

      {device.status !== "connected" ? (
        <p className="notice-hint">Connect this device to load its declared capabilities.</p>
      ) : null}
      {loading ? <LoadingNotice label="Querying capabilities…" /> : null}
      {error ? <ErrorNotice message={error} onRetry={() => void loadCapabilities()} /> : null}

      {capabilities ? (
        <>
          <table className="capabilities">
            <thead>
              <tr>
                <th scope="col">Feature</th>
                <th scope="col">Declared</th>
                <th scope="col">Detail</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.feature}>
                  <th scope="row">{row.feature}</th>
                  <td>{row.supported ? "yes" : "no"}</td>
                  <td>{row.detail}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {capabilities.lighting ? (
            <button type="button" onClick={() => onOpenLighting(device.interface_id)}>
              Open lighting controls
            </button>
          ) : null}
        </>
      ) : null}
    </section>
  );
}
