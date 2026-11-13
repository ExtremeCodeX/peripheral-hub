import { useSyncExternalStore } from 'react';
import { Icon } from './Icon';

const key = 'peripheral-hub.theme';
type Theme = 'light' | 'dark';
function savedTheme(): Theme {
  try { return window.localStorage.getItem(key) === 'light' ? 'light' : 'dark'; }
  catch { return 'dark'; }
}
document.documentElement.dataset.theme = savedTheme();

function subscribe(listener: () => void) {
  const onStorage = (event: StorageEvent) => {
    if (event.key === key || event.key === null) {
      document.documentElement.dataset.theme = savedTheme();
      listener();
    }
  };
  window.addEventListener('hub-theme-change', listener);
  window.addEventListener('storage', onStorage);
  return () => {
    window.removeEventListener('hub-theme-change', listener);
    window.removeEventListener('storage', onStorage);
  };
}

export function ThemeToggle() {
  const theme = useSyncExternalStore(subscribe, () => document.documentElement.dataset.theme ?? 'dark');
  const label = theme === 'dark' ? 'Ativar modo claro' : 'Ativar modo escuro';
  function toggle() {
    const next = theme === 'dark' ? 'light' : 'dark';
    document.documentElement.dataset.theme = next;
    try { window.localStorage.setItem(key, next); } catch { /* Still works for this session. */ }
    window.dispatchEvent(new Event('hub-theme-change'));
  }
  return <button className="icon-button theme-toggle" type="button" aria-label={label} title={label} onClick={toggle}><Icon name={theme === 'dark' ? 'sun' : 'moon'} /></button>;
}
