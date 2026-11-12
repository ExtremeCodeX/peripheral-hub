import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { StarterPreview } from './Preview';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }));
afterEach(() => cleanup());
beforeEach(() => { vi.clearAllMocks(); vi.mocked(isTauri).mockReturnValue(true); });
describe('starter real transport UI', () => {
  it('selects a named plugin and renders only its declared capabilities', async () => {
    const settings = [
      { key: 'gain', access: ['read', 'write'], effects: 'simulation_only', schema: { type: 'integer', enum: [1, 2, 3] } },
      { key: 'enabled', access: ['read'], effects: 'simulation_only', schema: { type: 'boolean' } },
    ];
    vi.mocked(invoke).mockImplementation(async (_command, args) => {
      const request = (args as { request: { action: string } }).request;
      if (request.action === 'capabilities') return { settings };
      if (request.action === 'open') return { session: 'a'.repeat(32) };
      if (request.action === 'read') return { mode: 'fake', hardwareAccess: false, outcome: 'simulated', session: 'a'.repeat(32), baselineToken: 'b'.repeat(32), values: { gain: 2, enabled: true } };
      return {};
    });
    render(<StarterPreview onBack={() => undefined} />);
    fireEvent.change(screen.getByLabelText('Integração local registrada'), { target: { value: 'dev.another-mouse' } });
    fireEvent.click(screen.getByRole('button', { name: 'Selecionar integração' }));
    expect(invoke).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Iniciar processo' }));
    await screen.findByText('Lido do processo: 2 · resultado simulado.');
    expect(screen.queryByText('Rascunho simulado')).toBeNull();
    expect(screen.getByRole('slider').getAttribute('min')).toBe('1');
    expect((screen.getByRole('checkbox') as HTMLInputElement).disabled).toBe(true);
    expect(vi.mocked(invoke).mock.calls.some(([, args]) => JSON.stringify(args).includes('dev.another-mouse'))).toBe(true);
  });
  it('renders only returned state; draft is not reported as applied before RPC', async () => {
    let value = 50;
    vi.mocked(invoke).mockImplementation(async (_command, args) => {
      const request = (args as { request: { action: string; value?: number } }).request;
      if (request.action === 'capabilities') return { settings: [{ key: 'sample_value', access: ['read', 'write'], effects: 'simulation_only', schema: { type: 'integer', enum: [0, 25, 50, 75, 100] } }] };
      if (request.action === 'open') return { session: 'a'.repeat(32) };
      if (request.action === 'set_setting') value = request.value!;
      if (request.action === 'read' || request.action === 'set_setting') return { mode: 'fake', hardwareAccess: false, outcome: 'simulated', session: 'a'.repeat(32), baselineToken: 'b'.repeat(32), values: { sample_value: value } };
      return {};
    });
    const back = vi.fn();
    render(<StarterPreview onBack={back} />);
    expect(document.activeElement).toBe(screen.getByRole('heading', { level: 2 }));
    const slider = screen.getByRole('slider') as HTMLInputElement;
    expect(slider.disabled).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Iniciar processo' }));
    await waitFor(() => expect(slider.disabled).toBe(false));
    expect(screen.getByText('Lido do processo: 50 · resultado simulado.')).toBeTruthy();
    fireEvent.change(slider, { target: { value: '75' } });
    expect(screen.getByText('Lido do processo: 50 · resultado simulado.')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar no simulador' }));
    await screen.findByText('Lido do processo: 75 · resultado simulado.');
    fireEvent.click(screen.getByRole('button', { name: 'Ler estado' }));
    await waitFor(() => expect(slider.disabled).toBe(false));
    fireEvent.click(screen.getByRole('button', { name: 'Parar processo' }));
    await screen.findByRole('button', { name: 'Iniciar processo' });
    expect(slider.disabled).toBe(true);
    expect(vi.mocked(invoke).mock.calls.every(([name]) => name === 'starter_local')).toBe(true);
    expect(vi.mocked(invoke).mock.calls.some(([, args]) => (args as { request: { action: string } }).request.action === 'set_setting')).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(back).toHaveBeenCalledOnce();
  });
  it('has no browser fallback or fabricated state', async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    render(<StarterPreview onBack={() => undefined} />);
    fireEvent.click(screen.getByRole('button', { name: 'Iniciar processo' }));
    expect((await screen.findByRole('alert')).textContent).toContain('No navegador e em release não há execução nem mock automático');
    expect((screen.getByRole('slider') as HTMLInputElement).disabled).toBe(true);
    expect(invoke).not.toHaveBeenCalled();
  });
});
