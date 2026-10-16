import { ErrorNotice } from "../../components/StateNotice";
import { ModeBadge } from "../../components/Badge";
import { useDaemon } from "../../hooks/useDaemon";
import { errorMessage } from "../../services/daemonController";
import { IPC_VERSION } from "../../types/ipc";
import { describeDeviceError } from "../../services/errors";

export function DiagnosticsScreen() {
  const { state, controller } = useDaemon();
  const health = state.health;

  return (
    <section>
      <div className="section-head">
        <h2>Diagnostics</h2>
        <button
          type="button"
          onClick={() => {
            void controller.refreshHealth().catch((cause) => {
              // Refresh failures are surfaced through the connection banner.
              console.warn(errorMessage(cause));
            });
          }}
        >
          Refresh health
        </button>
      </div>

      {state.connection === "error" ? <ErrorNotice message={state.error ?? "Daemon unavailable."} /> : null}

      {health ? (
        <>
          <div className="detail-meta">
            <ModeBadge mode={health.mode} />
          </div>
          <dl className="facts">
            <dt>Daemon version</dt>
            <dd>{health.daemon_version}</dd>
            <dt>SDK contract</dt>
            <dd>
              {health.sdk_version.major}.{health.sdk_version.minor}
            </dd>
            <dt>IPC version</dt>
            <dd>{IPC_VERSION}</dd>
            <dt>Session id</dt>
            <dd>{health.session_id}</dd>
            <dt>Event sequence</dt>
            <dd>{health.sequence}</dd>
            <dt>Discovery</dt>
            <dd>
              {health.discovery_error
                ? describeDeviceError(health.discovery_error)
                : "Last enumeration succeeded"}
            </dd>
          </dl>
        </>
      ) : (
        <p className="notice-hint">No health information yet.</p>
      )}

      <h3>Requesting a new device</h3>
      <p className="notice-hint">
        To add support for a peripheral, follow the contributor guide: identity and authorized
        evidence first, then a driver, then validation. Discovery alone is not a compatibility claim.
      </p>

      <h3>Device Research Assistant (DRA)</h3>
      <p className="notice-hint">
        The DRA is a standalone research CLI and is not part of the shipped runtime. It is documented
        for authorized, non-destructive protocol study only.
      </p>
    </section>
  );
}
