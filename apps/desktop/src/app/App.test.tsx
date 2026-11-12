import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { App } from './App';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
beforeAll(() => {
  Object.defineProperties(HTMLDialogElement.prototype, {
    showModal: { configurable: true, value() { this.setAttribute('open', ''); } },
    close: { configurable: true, value() { this.removeAttribute('open'); } },
  });
});
beforeEach(() => { vi.spyOn(window, 'scrollTo').mockImplementation(() => {}); });
afterEach(() => { cleanup(); vi.clearAllMocks(); });

function openMenu() { fireEvent.click(screen.getByRole('button', { name: 'Menu do aplicativo' })); }
function renderExplore() {
  render(<App />);
  openMenu();
  fireEvent.click(screen.getByRole('button', { name: 'Exemplos para desenvolvedores' }));
}
function openDetails(name = 'NOVA AIR') { fireEvent.click(screen.getByRole('button', { name: `Ver detalhes de ${name}` })); }

describe('coleção simplificada', () => {
  it('registers the starter locally without auto-start and returns focus to its library card', () => {
    render(<App software={[]} />);
    openMenu();
    fireEvent.click(screen.getByRole('button', { name: 'Registrar starter local (DEV)' }));
    const card = screen.getByRole('button', { name: 'Abrir Starter local — simulado' });
    fireEvent.click(card);
    expect(screen.getByRole('heading', { name: 'Starter local — simulado' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Abrir Starter local — simulado' }));
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === 'plugin_poc')).toBe(false);
  });
  it('mantém o nome acessível sem ícone no cabeçalho', () => {
    renderExplore();
    const heading = screen.getByRole('heading', { name: 'Peripheral Hub' });
    expect(heading.querySelector('img')).toBeNull();
    expect(heading.querySelector('.wordmark__dot')?.getAttribute('aria-hidden')).toBe('true');
  });

  it('usa o cartão inteiro para abrir detalhes, com fabricante e estado no rodapé', () => {
    renderExplore();
    const card = screen.getByRole('button', { name: 'Ver detalhes de AURA M1' });
    expect(card.querySelector('.device-card__manufacturer')?.textContent).toBe('Aster');
    expect(card.querySelector('.device-card__footer')?.textContent).toContain('Completo');
    expect(card.querySelector('.device-card__header .software-status')).toBeNull();
    expect(card.getAttribute('aria-pressed')).toBeNull();
    expect(card.querySelector('.device-card__arrow')?.getAttribute('aria-hidden')).toBe('true');
    expect(card.querySelector('button')).toBeNull();
    expect(card.textContent).not.toContain('Gelo');
    expect(card.textContent).not.toContain('Forma simétrica');
    fireEvent.click(card.querySelector('.mouse-artwork')!);
    expect(screen.getByRole('heading', { name: 'AURA M1' })).toBeTruthy();
    expect(invoke).not.toHaveBeenCalled();
  });

  it('mostra cobertura e BETA antes de abrir a tela, sem seletores intermediários', () => {
    renderExplore();
    expect(screen.getAllByRole('button', { name: /^Ver detalhes de / })).toHaveLength(6);
    expect(screen.getByRole('button', { name: 'Ver detalhes de NOVA AIR' }).textContent).toContain('CompletoBETA');
    expect(screen.getByRole('button', { name: 'Ver detalhes de FORMA ERGO' }).textContent).toContain('ParcialBETA');
    expect(screen.queryByRole('button', { name: /^Selecionar / })).toBeNull();
    expect(screen.getByText('Modo demonstração')).toBeTruthy();
    expect(document.querySelector('button button')).toBeNull();
  });

  it('pesquisa modelo ou fabricante ignorando acentos, não acabamento cosmético', () => {
    renderExplore();
    const search = screen.getByRole('searchbox');
    fireEvent.change(search, { target: { value: 'nova' } });
    expect(screen.getAllByRole('button', { name: /^Ver detalhes de / })).toHaveLength(1);
    fireEvent.change(search, { target: { value: ' ORBITA ' } });
    expect(screen.getAllByRole('button', { name: /^Ver detalhes de / })).toHaveLength(2);
    fireEvent.change(search, { target: { value: 'Gelo' } });
    expect(screen.getByRole('heading', { name: 'Nenhum mouse por aqui.' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Ver todos os exemplos' }));
    expect(screen.getAllByRole('button', { name: /^Ver detalhes de / })).toHaveLength(6);
    expect(document.activeElement).toBe(search);
  });

  it('mantém Escape na busca e Ctrl+K', () => {
    renderExplore();
    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'Nimbo' } });
    fireEvent.keyDown(screen.getByRole('searchbox'), { key: 'Escape' });
    expect((screen.getByRole('searchbox') as HTMLInputElement).value).toBe('');
    fireEvent.keyDown(document, { key: 'k', ctrlKey: true });
    expect(document.activeElement).toBe(screen.getByRole('searchbox'));
  });

  it('fecha o menu com Escape e clique fora', () => {
    renderExplore();
    openMenu();
    expect(document.activeElement).toBe(within(screen.getByRole('region', { name: 'Opções do aplicativo' })).getByRole('button', { name: 'Meus softwares' }));
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Menu do aplicativo' }));
    openMenu();
    fireEvent.pointerDown(screen.getByRole('heading', { name: 'Peripheral Hub' }));
    expect(screen.queryByRole('region', { name: 'Opções do aplicativo' })).toBeNull();
  });

  it('oferece página de exemplos com um caso por combinação relevante', () => {
    renderExplore();
    openMenu();
    fireEvent.click(screen.getByRole('button', { name: 'Exemplos para desenvolvedores' }));
    expect(screen.getByRole('heading', { name: 'Exemplos para desenvolvedores' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Software diferente por cor' })).toBeTruthy();
    expect(screen.getAllByRole('button', { name: /^Ver detalhes de / })).toHaveLength(6);
    openDetails('ARC PRO');
    fireEvent.click(screen.getByRole('button', { name: 'Voltar aos exemplos' }));
    expect(screen.getByRole('heading', { name: 'Exemplos para desenvolvedores' })).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Ver detalhes de ARC PRO' }));
    openMenu();
    fireEvent.click(within(screen.getByRole('region', { name: 'Opções do aplicativo' })).getByRole('button', { name: 'Meus softwares' }));
    expect(screen.queryByRole('heading', { name: 'Software diferente por cor' })).toBeNull();
  });

  it('preserva o diagnóstico Tauri e o tratamento de falha', async () => {
    vi.mocked(invoke).mockResolvedValueOnce('Backend Rust disponível.');
    renderExplore();
    openMenu();
    fireEvent.click(screen.getByRole('button', { name: 'Verificar backend' }));
    expect(await screen.findByText('Backend Rust disponível.')).toBeTruthy();
    vi.mocked(invoke).mockRejectedValueOnce(new Error('IPC indisponível'));
    fireEvent.click(screen.getByRole('button', { name: 'Verificar backend' }));
    expect(await screen.findByText(/Abra o aplicativo desktop e tente novamente/)).toBeTruthy();
  });
});

describe('detalhes enxutos e variantes', () => {
  it('mantém versão/criador sem resumos, legenda de cor ou bloco redundante de limites', () => {
    renderExplore();
    openDetails('VECTOR S');
    expect(screen.getByRole('heading', { name: 'VECTOR S' })).toBeTruthy();
    expect(screen.getByRole('button', { name: /0.6.0-demo — repositório não vinculado/ })).toBeTruthy();
    expect((screen.getByRole('button', { name: /Criador de exemplo — perfil não vinculado/ }) as HTMLButtonElement).disabled).toBe(true);
    expect(document.querySelector('.details-description')).toBeNull();
    expect(document.querySelector('.scope-note')).toBeNull();
    expect(document.querySelector('.section-description')).toBeNull();
    expect(document.querySelector('.details-artwork__caption')).toBeNull();
    expect(screen.getAllByText('Não implementado')).toHaveLength(2);
    expect(screen.getByText('Revisada no exemplo')).toBeTruthy();
  });

  it('preserva filtro e foco ao voltar pelo cartão', () => {
    renderExplore();
    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'Nimbo' } });
    openDetails();
    fireEvent.click(screen.getByRole('button', { name: 'Voltar aos exemplos' }));
    expect((screen.getByRole('searchbox') as HTMLInputElement).value).toBe('Nimbo');
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Ver detalhes de NOVA AIR' }));
  });

  it('não oferece cor nos modelos sem diferença documentada', () => {
    renderExplore();
    for (const name of ['AURA M1', 'VECTOR S', 'NOVA AIR', 'FORMA ERGO', 'PULSE MINI']) {
      openDetails(name);
      expect(screen.queryByRole('group', { name: 'Cor / variante do hardware' })).toBeNull();
      fireEvent.click(screen.getByRole('button', { name: 'Voltar aos exemplos' }));
    }
  });

  it('altera software, funções, estado e prompt conforme a variante excepcional', () => {
    renderExplore();
    openDetails('ARC PRO');
    expect(screen.getByRole('group', { name: 'Cor / variante do hardware' })).toBeTruthy();
    expect(screen.getByText(/Areia e Grafite usam eletrônicas diferentes/)).toBeTruthy();
    expect(screen.getByText('Parcial')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Grafite' }));
    expect(screen.getByText('0.5.0-demo')).toBeTruthy();
    expect(screen.getByText('Completo')).toBeTruthy();
    expect(screen.queryByText('Não implementado')).toBeNull();
    expect(screen.getByText('1 de 2 aprovações')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Avalie esta integração' }));
    fireEvent.click(screen.getByRole('button', { name: 'Ver prompt' }));
    const prompt = screen.getByLabelText('Conteúdo do prompt de avaliação').textContent;
    expect(prompt).toContain('Cor / variante: Grafite');
    expect(prompt).toContain('0.5.0-demo');
    expect(prompt).toContain('avaliacao-arc-grafite.md');
  });

  it('retorna à busca sem roubar foco e mantém cobertura separada de BETA', () => {
    renderExplore();
    openDetails();
    expect(screen.getByText('Completo')).toBeTruthy();
    expect(screen.getByText('0 de 2 aprovações')).toBeTruthy();
    const search = screen.getByRole('searchbox');
    search.focus();
    fireEvent.change(search, { target: { value: 'vector' } });
    expect(screen.getByRole('button', { name: 'Ver detalhes de VECTOR S' })).toBeTruthy();
    expect(document.activeElement).toBe(search);
  });
});

describe('avaliação preservada', () => {
  it('começa com o prompt oculto, retorna o foco e não altera aprovações', () => {
    renderExplore();
    openDetails('FORMA ERGO');
    const trigger = screen.getByRole('button', { name: 'Avalie esta integração' });
    fireEvent.click(trigger);
    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(screen.queryByLabelText('Conteúdo do prompt de avaliação')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Fechar avaliação' }));
    expect(document.activeElement).toBe(trigger);
    expect(screen.getByText('1 de 2 aprovações')).toBeTruthy();
    expect(invoke).not.toHaveBeenCalled();
  });

  it('usa um só modal, prévia opcional e cancelamento nativo', () => {
    renderExplore();
    openDetails();
    const beta = screen.getByRole('button', { name: 'Entender a validação BETA' });
    fireEvent.click(beta);
    fireEvent.click(screen.getByRole('button', { name: 'Ver prompt' }));
    expect(screen.getAllByRole('dialog')).toHaveLength(1);
    expect(screen.getByLabelText('Conteúdo do prompt de avaliação').textContent).toContain('NOVA AIR');
    fireEvent.click(screen.getByRole('button', { name: 'Voltar às instruções' }));
    expect(screen.queryByLabelText('Conteúdo do prompt de avaliação')).toBeNull();
    fireEvent(screen.getByRole('dialog'), new Event('cancel', { cancelable: true }));
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(document.activeElement).toBe(beta);
  });

  it('copia o prompt e trata falha de cópia na prévia', async () => {
    const writeText = vi.fn().mockResolvedValueOnce(undefined).mockRejectedValueOnce(new Error('negado'));
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    Object.defineProperty(document, 'execCommand', { configurable: true, value: vi.fn().mockReturnValue(false) });
    renderExplore();
    openDetails();
    fireEvent.click(screen.getByRole('button', { name: 'Avalie esta integração' }));
    fireEvent.click(screen.getByRole('button', { name: 'Copiar prompt de avaliação' }));
    expect(await screen.findByText(/Prompt copiado/)).toBeTruthy();
    expect(writeText.mock.calls[0][0]).toContain('NOVA AIR');
    fireEvent.click(screen.getByRole('button', { name: 'Ver prompt' }));
    fireEvent.click(screen.getByRole('button', { name: 'Copiar novamente' }));
    expect(await screen.findByText(/Selecione o texto exibido acima/)).toBeTruthy();
    expect(document.querySelector('textarea')).toBeNull();
  });
});
