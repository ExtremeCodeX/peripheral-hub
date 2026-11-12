import { afterEach, describe, expect, it, vi } from 'vitest';
import { copyText } from './copy-text';

afterEach(() => { document.body.innerHTML = ''; vi.restoreAllMocks(); });

describe('cópia local', () => {
  it('usa a API do clipboard quando disponível', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    await copyText('teste');
    expect(writeText).toHaveBeenCalledWith('teste');
  });

  it('usa fallback local, remove campo temporário e preserva foco', async () => {
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: undefined });
    const execCommand = vi.fn().mockReturnValue(true);
    Object.defineProperty(document, 'execCommand', { configurable: true, value: execCommand });
    const button = document.createElement('button');
    document.body.append(button);
    button.focus();
    await copyText('teste');
    expect(execCommand).toHaveBeenCalledWith('copy');
    expect(document.querySelector('textarea')).toBeNull();
    expect(document.activeElement).toBe(button);
  });

  it('não anuncia sucesso se os dois métodos falham', async () => {
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: vi.fn().mockRejectedValue(new Error('negado')) } });
    Object.defineProperty(document, 'execCommand', { configurable: true, value: vi.fn().mockReturnValue(false) });
    await expect(copyText('teste')).rejects.toThrow('Clipboard indisponível');
    expect(document.querySelector('textarea')).toBeNull();
  });
});
