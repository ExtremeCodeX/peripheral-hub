import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

/** Window controls only: no shell, process, filesystem or device APIs. */
export function WindowBar() {
  const native = '__TAURI_INTERNALS__' in window;
  const [maximized, setMaximized] = useState(false);
  const [error, setError] = useState('');
  useEffect(() => {
    if (!native) return;
    let alive = true;
    let unlisten: (() => void) | undefined;
    const win = getCurrentWindow();
    const update = async () => {
      try { const value = await win.isMaximized(); if (alive) setMaximized(value); }
      catch { if (alive) setError('Não foi possível consultar a janela.'); }
    };
    void update();
    void win.onResized(() => { void update(); }).then((off) => { if (alive) unlisten = off; else off(); }).catch(() => { if (alive) setError('Não foi possível acompanhar a janela.'); });
    return () => { alive = false; unlisten?.(); };
  }, [native]);
  if (!native) return null;
  async function perform(action: 'minimize' | 'toggleMaximize' | 'close' | 'startDragging') {
    setError('');
    try {
      const win = getCurrentWindow();
      await win[action]();
      if (action === 'toggleMaximize') setMaximized(await win.isMaximized());
    } catch { setError('Não foi possível controlar a janela. Tente novamente ou use os atalhos do gerenciador de janelas.'); }
  }
  return <header className="window-bar" aria-label="Janela do Peripheral Hub">
    <div className="window-bar__drag" onPointerDown={(event) => { if (event.button === 0 && event.isPrimary && event.detail < 2) void perform('startDragging'); }} onDoubleClick={() => { void perform('toggleMaximize'); }}><span>Peripheral Hub</span></div>
    <div className="window-bar__controls"><button type="button" aria-label="Minimizar janela" onClick={() => { void perform('minimize'); }}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 16h14" /></svg></button><button type="button" aria-label={maximized ? 'Restaurar janela' : 'Maximizar janela'} onClick={() => { void perform('toggleMaximize'); }}><svg viewBox="0 0 24 24" aria-hidden="true">{maximized ? <path d="M8 8V5h11v11h-3M5 8h11v11H5z" /> : <path d="M5 5h14v14H5z" />}</svg></button><button type="button" aria-label="Fechar janela" onClick={() => { void perform('close'); }}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18" /></svg></button></div>
    {error && <p className="window-bar__error" role="alert">{error}</p>}
  </header>;
}
