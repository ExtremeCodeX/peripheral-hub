import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { ThemeToggle } from './ThemeToggle';

beforeEach(() => {
  const values = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    getItem: vi.fn((key: string) => values.get(key) ?? null),
    setItem: vi.fn((key: string, value: string) => values.set(key, value)),
  });
  document.documentElement.dataset.theme = 'dark';
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

it('alterna com um clique, persiste e mantém instâncias sincronizadas', () => {
  render(<><ThemeToggle /><ThemeToggle /></>);
  fireEvent.click(screen.getAllByRole('button', { name: 'Ativar modo claro' })[0]);
  expect(document.documentElement.dataset.theme).toBe('light');
  expect(window.localStorage.getItem('peripheral-hub.theme')).toBe('light');
  expect(screen.getAllByRole('button', { name: 'Ativar modo escuro' })).toHaveLength(2);
  fireEvent.click(screen.getAllByRole('button', { name: 'Ativar modo escuro' })[1]);
  expect(document.documentElement.dataset.theme).toBe('dark');
});

it('continua funcionando se o armazenamento falhar', () => {
  vi.mocked(window.localStorage.setItem).mockImplementation(() => { throw new Error('blocked'); });
  render(<ThemeToggle />);
  fireEvent.click(screen.getByRole('button', { name: 'Ativar modo claro' }));
  expect(document.documentElement.dataset.theme).toBe('light');
});

it('sincroniza preferências salvas em outra janela', () => {
  render(<ThemeToggle />);
  window.localStorage.setItem('peripheral-hub.theme', 'light');
  act(() => window.dispatchEvent(new StorageEvent('storage', { key: 'peripheral-hub.theme' })));
  expect(screen.getByRole('button', { name: 'Ativar modo escuro' })).toBeTruthy();
});
