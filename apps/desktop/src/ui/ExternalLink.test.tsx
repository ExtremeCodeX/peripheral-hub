import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { ExternalLink, isSafeExternalUrl } from './ExternalLink';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

describe('links de origem', () => {
  it('aceita somente HTTPS sem credenciais ou controles', () => {
    expect(isSafeExternalUrl('https://example.org/profile')).toBe(true);
    for (const url of [null, '', 'file:///etc/passwd', 'javascript:alert(1)', 'http://example.org', 'https://a:b@example.org', 'https://example.org/\n']) expect(isSafeExternalUrl(url)).toBe(false);
  });

  it('não inventa navegação quando falta o link', () => {
    render(<ExternalLink label="Criador de exemplo" url={null} kind="perfil" />);
    const button = screen.getByRole('button', { name: /perfil não vinculado/ });
    expect((button as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(button);
    expect(invoke).not.toHaveBeenCalled();
  });

  it('solicita ao comando nativo somente o URL após clique explícito', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    render(<ExternalLink label="1.0.0" url="https://example.org/releases/1.0.0" kind="repositório" />);
    expect(invoke).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button'));
    expect(invoke).toHaveBeenCalledWith('open_external_url', { url: 'https://example.org/releases/1.0.0' });
    await waitFor(() => expect((screen.getByRole('button') as HTMLButtonElement).disabled).toBe(false));
  });

  it('informa erro em vez de fingir redirecionamento', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('sem navegador'));
    render(<ExternalLink label="Pessoa Exemplo" url="https://example.org/profile" kind="perfil" />);
    fireEvent.click(screen.getByRole('button'));
    expect(await screen.findByText(/Não foi possível abrir o link/)).toBeTruthy();
  });
});
