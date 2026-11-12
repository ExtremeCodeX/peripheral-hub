import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { WindowBar } from './WindowBar';

const win = vi.hoisted(() => ({ isMaximized: vi.fn(), onResized: vi.fn(), minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn(), startDragging: vi.fn() }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => win }));
beforeEach(() => {
  Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} });
  win.isMaximized.mockResolvedValue(false);
  win.onResized.mockResolvedValue(vi.fn());
});
afterEach(() => { cleanup(); Reflect.deleteProperty(window, '__TAURI_INTERNALS__'); vi.resetAllMocks(); });

it('não mostra controles nativos no navegador', () => {
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__');
  render(<WindowBar />);
  expect(screen.queryByRole('button')).toBeNull();
  expect(win.onResized).not.toHaveBeenCalled();
});
it('minimiza, maximiza/restaura e fecha pelas APIs da janela', async () => {
  render(<WindowBar />);
  await act(async () => {});
  fireEvent.click(screen.getByRole('button', { name: 'Minimizar janela' }));
  await waitFor(() => expect(win.minimize).toHaveBeenCalledTimes(1));
  win.isMaximized.mockResolvedValue(true);
  fireEvent.click(screen.getByRole('button', { name: 'Maximizar janela' }));
  await screen.findByRole('button', { name: 'Restaurar janela' });
  expect(win.toggleMaximize).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole('button', { name: 'Fechar janela' }));
  await waitFor(() => expect(win.close).toHaveBeenCalledTimes(1));
  expect(win.startDragging).not.toHaveBeenCalled();
});
it('arrasta só a região livre com ponteiro primário e alterna por duplo clique', async () => {
  const view = render(<WindowBar />);
  await act(async () => {});
  const drag = view.container.querySelector('.window-bar__drag')!;
  const event = new Event('pointerdown', { bubbles: true });
  Object.assign(event, { button: 0, isPrimary: true, detail: 1 });
  fireEvent(drag, event);
  await waitFor(() => expect(win.startDragging).toHaveBeenCalledTimes(1));
  fireEvent.doubleClick(drag);
  await waitFor(() => expect(win.toggleMaximize).toHaveBeenCalledTimes(1));
});
it('limpa listener de resize mesmo quando chega depois da desmontagem', async () => {
  let finish!: (off: () => void) => void;
  const off = vi.fn();
  win.onResized.mockReturnValue(new Promise((resolve) => { finish = resolve; }));
  const view = render(<WindowBar />);
  view.unmount();
  await act(async () => finish(off));
  expect(off).toHaveBeenCalledTimes(1);
});
it('mostra falha de comando sem fingir sucesso', async () => {
  win.minimize.mockRejectedValueOnce(new Error('denied'));
  render(<WindowBar />);
  await act(async () => {});
  fireEvent.click(screen.getByRole('button', { name: 'Minimizar janela' }));
  expect((await screen.findByRole('alert')).textContent).toContain('Não foi possível controlar a janela');
});
