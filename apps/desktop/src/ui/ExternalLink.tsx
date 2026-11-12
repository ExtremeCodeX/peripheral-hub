import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Icon } from './Icon';

export function isSafeExternalUrl(value: string | null): value is string {
  if (!value || /[\u0000-\u001f\u007f-\u009f]/.test(value)) return false;
  try {
    const url = new URL(value);
    return url.protocol === 'https:' && !!url.hostname && !url.username && !url.password;
  } catch { return false; }
}

export function ExternalLink({ label, url, kind }: { label: string; url: string | null; kind: 'perfil' | 'repositório' }) {
  const [error, setError] = useState(false);
  const [opening, setOpening] = useState(false);
  const enabled = isSafeExternalUrl(url);

  async function open() {
    if (!enabled || opening) return;
    setOpening(true);
    setError(false);
    try { await invoke('open_external_url', { url }); } catch { setError(true); } finally { setOpening(false); }
  }

  return <span className="metadata-action">
    <button className="metadata-link" type="button" disabled={!enabled || opening} aria-label={`${label} — ${enabled ? `abrir ${kind} no navegador` : `${kind} não vinculado`}`} title={enabled ? `Abrir ${kind} no navegador padrão` : `Sem ${kind} vinculado neste exemplo`} onClick={open}>{label}<Icon name="external" /></button>
    {error && <span className="metadata-error" role="status">Não foi possível abrir o link. Tente novamente no aplicativo desktop.</span>}
  </span>;
}
