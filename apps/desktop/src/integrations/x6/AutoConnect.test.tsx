import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { X6Control } from './Control';
import { starterClient } from '../starter/client';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('../starter/client', () => ({
  starterClient: { start: vi.fn(), stop: vi.fn().mockResolvedValue({}), read: vi.fn(), set: vi.fn() },
  starterError: () => 'Não foi possível conectar.',
}));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('connects on entry and lets the user retry a failed connection without writing settings', async () => {
  vi.mocked(starterClient.start).mockRejectedValueOnce(new Error('DEVICE_NOT_FOUND')).mockResolvedValueOnce({
    mode: 'hardware', hardwareAccess: true, outcome: 'read_from_device', session: 'a', baselineToken: 'b',
    values: { dpi_stage: 1, polling_rate_hz: 1000 }, settings: [],
  });
  render(<X6Control onBack={() => undefined} pluginId="io.github.extremecodex.attack-shark-x6" />);
  expect((await screen.findByRole('alert')).textContent).toContain('Não foi possível conectar.');
  expect(starterClient.start).toHaveBeenCalledWith('io.github.extremecodex.attack-shark-x6', true);
  fireEvent.click(screen.getByRole('button', { name: 'Tentar novamente' }));
  await waitFor(() => expect(screen.getByText('CONECTADO')).toBeTruthy());
  expect(starterClient.set).not.toHaveBeenCalled();
});

it('shows permission setup instructions when Linux denies access', async () => {
  vi.mocked(starterClient.start).mockRejectedValueOnce({ kind: 'PERMISSION_DENIED' });
  render(<X6Control onBack={() => undefined} />);
  expect(await screen.findByRole('heading', { name: 'Permissão necessária para acessar o mouse' })).toBeTruthy();
  expect(screen.getByText(/sudo udevadm control --reload-rules/)).toBeTruthy();
  expect(screen.getByText(/outros receptores com os mesmos identificadores/)).toBeTruthy();
  expect(starterClient.set).not.toHaveBeenCalled();
});
