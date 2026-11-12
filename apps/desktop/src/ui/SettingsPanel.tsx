import type { ReactNode } from 'react';
import './settings-panel.css';

/** Visual grouping only; the caller owns status, controls and device operations. */
export function SettingsPanel({ id, title, status, children, action }: { id: string; title: string; status: string; children: ReactNode; action?: ReactNode }) {
  return <section className="settings-panel" aria-labelledby={id}>
    <div className="settings-panel__heading"><h3 id={id}>{title}</h3><span>{status}</span>{action}</div>
    {children}
  </section>;
}
