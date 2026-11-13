import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { X6Control } from './Control';
import { starterClient, type Snapshot } from '../starter/client';
import { fromHex, toHex } from './Lighting';

vi.mock('../starter/client', async (original) => {
  const actual = await original<typeof import('../starter/client')>();
  return { ...actual, starterClient: { start: vi.fn(), stop: vi.fn(), read: vi.fn(), set: vi.fn() } };
});
function snapshot(): Snapshot {
  return { mode: 'hardware', hardwareAccess: true, outcome: 'read_from_device', session: 'a'.repeat(32), baselineToken: 'b'.repeat(32),
    values: { button_1: 'left_click', button_2: 'right_click', button_3: 'middle_click', button_4: 'key_2', button_5: 'key_3', button_6: 'dpi_up', button_7: 'dpi_down', dpi_stage: 4, dpi_stage_1: 800, dpi_stage_2: 1200, dpi_stage_3: 1600, dpi_stage_4: 3200, dpi_stage_5: 5600, dpi_stage_6: 26000, polling_rate_hz: 1000, battery_raw: 10, connection: 'connected_receiver' },
    settings: [1, 2, 3, 4, 5, 6, 7].map((n) => ({ key: `button_${n}`, access: n >= 4 ? ['read', 'write'] : ['read'], effects: n >= 4 ? 'active_read_and_configuration_write' : 'active_read', schema: { type: 'string', enum: ['left_click', 'right_click', 'middle_click', 'key_a', 'key_2', 'key_3', 'key_f5', 'key_space', 'key_left_ctrl', 'combo_ctrl_c', 'media_volume_up', 'dpi_cycle', 'dpi_up', 'dpi_down', 'restore_session'] } })),
    device: { candidateId: 'c'.repeat(32), vid: '1d57', pid: 'fa60', transport: 'usb_receiver', revision: 'descriptor_matched', interface: '2', hidraw: 'hidraw_fixture', identity: 'unconfirmed' } };
}
afterEach(cleanup);
function rgbSnapshot(): Snapshot {
  const state = snapshot();
  Object.assign(state.values, { dpi_color_1: 0xff0000, dpi_color_2: 0x00ff00, dpi_color_3: 255, dpi_color_4: 0xffff00, dpi_color_5: 65535, dpi_color_6: 0xff00ff, lighting_mode: 'static_dpi', lighting_color: 0x00ff00, lighting_brightness: 8, lighting_speed: 3 });
  state.settings.push(...['dpi_color_1', 'dpi_color_2', 'dpi_color_3', 'dpi_color_4', 'dpi_color_5', 'dpi_color_6', 'lighting_color', 'lighting_brightness', 'lighting_speed'].map((key): Snapshot['settings'][number] => ({ key, access: ['read', 'write'], effects: 'active_read_and_configuration_write', schema: { type: 'integer' } })));
  state.settings.push({ key: 'lighting_mode', access: ['read', 'write'], effects: 'active_read_and_configuration_write', schema: { type: 'string', enum: ['off', 'fixed', 'breathing', 'neon', 'color_breathing', 'static_dpi', 'breathing_dpi'] } });
  return state;
}
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(starterClient.start).mockResolvedValue(snapshot());
  vi.mocked(starterClient.stop).mockResolvedValue({});
  vi.mocked(starterClient.read).mockResolvedValue(snapshot());
  vi.mocked(starterClient.set).mockImplementation(async (state, value, key) => ({ ...state, outcome: 'confirmed_by_readback', values: { ...state.values, [key!]: value === 'restore_session' ? snapshot().values[key!] : value } }));
});
async function connect() {
  fireEvent.click(screen.getByRole('button', { name: 'Conectar 2.4 GHz' }));
  await screen.findByText('HARDWARE · CONECTADO');
}
describe('X6 Control physical UI', () => {
  it('converts strict RGB hexadecimal values without accepting partial/invalid colors', () => {
    expect(toHex(0xa855f7)).toBe('#A855F7'); expect(toHex(0)).toBe('#000000');
    expect(fromHex('#a855f7')).toBe(0xa855f7);
    for (const invalid of ['#FFF', '#12345Z', '#12345678', '123456']) expect(fromHex(invalid)).toBeNull();
    expect(toHex(-1)).toBe(''); expect(toHex(0x1000000)).toBe('');
  });
  it('applies only an explicit stage color through hardware RPC and restores the session original', async () => {
    vi.mocked(starterClient.start).mockResolvedValue(rgbSnapshot());
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.click(screen.getByRole('tab', { name: 'Iluminação' }));
    expect((screen.getByRole('combobox', { name: 'Estágio de DPI para cor' }) as HTMLSelectElement).value).toBe('4');
    expect(screen.getByRole('option', { name: 'Estágio 4 — 3200 DPI — ativo' })).toBeTruthy();
    fireEvent.change(screen.getByRole('textbox', { name: 'Hexadecimal — Estágio 4' }), { target: { value: '#ZZZZZZ' } });
    expect(screen.getByRole('button', { name: 'Aplicar cor — Estágio 4' }).hasAttribute('disabled')).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Roxo — Estágio 4' }));
    expect(screen.getByRole('img', { name: 'Prévia — Estágio 4: #A855F7' })).toBeTruthy();
    expect(starterClient.set).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar cor — Estágio 4' }));
    await screen.findAllByText(/Aplicar cor — Estágio 4 · Escrita confirmada/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(), 0xa855f7, 'dpi_color_4');
    fireEvent.click(screen.getByRole('button', { name: 'Restaurar cor original — Estágio 4' }));
    await screen.findAllByText(/Restaurar cor — Estágio 4 · Escrita confirmada/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(), 0xffff00, 'dpi_color_4');
    fireEvent.change(screen.getByRole('combobox', { name: 'Estágio de DPI para cor' }), { target: { value: '2' } });
    expect((screen.getByRole('textbox', { name: 'Hexadecimal — Estágio 2' }) as HTMLInputElement).value).toBe('#00FF00');
    expect(starterClient.set).toHaveBeenCalledTimes(2);
  });
  it('keeps effects explicit, gates custom RGB by the read mode and writes shared levels separately', async () => {
    vi.mocked(starterClient.start).mockResolvedValue(rgbSnapshot());
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.click(screen.getByRole('tab', { name: 'Iluminação' }));
    expect(screen.getByRole('button', { name: 'Aplicar cor — Efeito' }).hasAttribute('disabled')).toBe(true);
    const modes = screen.getByRole('combobox', { name: 'Modo de iluminação' });
    expect(modes.querySelectorAll('option')).toHaveLength(8);
    fireEvent.change(modes, { target: { value: 'fixed' } });
    expect(starterClient.set).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Aplicar cor — Efeito' }).hasAttribute('disabled')).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar efeito' }));
    await screen.findAllByText(/Aplicar modo de iluminação · Escrita confirmada/);
    fireEvent.change(screen.getByLabelText('Seletor RGB — Efeito'), { target: { value: '#123456' } });
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar cor — Efeito' }));
    await screen.findAllByText(/Aplicar cor — Efeito · Escrita confirmada/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(), 0x123456, 'lighting_color');
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Brilho' }), { target: { value: '9' } });
    expect(screen.getByRole('button', { name: 'Aplicar Brilho' }).hasAttribute('disabled')).toBe(true);
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Brilho' }), { target: { value: '4' } });
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar Brilho' }));
    await screen.findAllByText(/Aplicar Brilho · Escrita confirmada/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(), 4, 'lighting_brightness');
  });
  it('disables lighting on missing capabilities and clears state after a real ACK error', async () => {
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.click(screen.getByRole('tab', { name: 'Iluminação' }));
    expect(screen.getByRole('button', { name: 'Aplicar efeito' }).hasAttribute('disabled')).toBe(true);
    expect(screen.getByRole('textbox', { name: 'Hexadecimal — Estágio 4' }).hasAttribute('disabled')).toBe(true);
    expect(starterClient.set).not.toHaveBeenCalled();
    cleanup(); vi.mocked(starterClient.start).mockResolvedValue(rgbSnapshot());
    vi.mocked(starterClient.set).mockRejectedValue({ kind: 'ACK_FAILED', configurationOutcome: 'unknown' });
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.click(screen.getByRole('tab', { name: 'Iluminação' }));
    fireEvent.click(screen.getByRole('button', { name: 'Azul — Estágio 4' }));
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar cor — Estágio 4' }));
    expect((await screen.findByRole('alert')).textContent).toContain('ACK ausente ou rejeitado');
    expect(screen.getByRole('button', { name: 'Aplicar cor — Estágio 4' }).hasAttribute('disabled')).toBe(true);
  });
  it('searches friendly labels/categories and applies the actual F5 binding on all four secondary buttons', async () => {
    render(<X6Control onBack={() => undefined}/>); await connect();
    for (const [query,label] of [['F5','F5'],['Ctrl','Ctrl esquerdo'],['Espaço','Espaço'],['Volume','Volume +']]) {
      fireEvent.change(screen.getByRole('searchbox',{name:'Pesquisar função'}),{target:{value:query}});
      expect(screen.getByRole('option',{name:label})).toBeTruthy();
    }
    fireEvent.change(screen.getByRole('searchbox',{name:'Pesquisar função'}),{target:{value:''}});
    fireEvent.change(screen.getByRole('combobox',{name:'Categoria de função'}),{target:{value:'Teclas F'}});
    expect(screen.getByRole('option',{name:/F13.*firmware/}).hasAttribute('disabled')).toBe(true);
    for (const name of ['Lateral dianteiro','Lateral traseiro','Botão superior 1 (frente)','Botão superior 2 (trás)']) {
      const n=['Lateral dianteiro','Lateral traseiro','Botão superior 1 (frente)','Botão superior 2 (trás)'].indexOf(name)+4;
      fireEvent.click(screen.getByRole('button',{name:`Selecionar ${name} no mouse`}));
      fireEvent.change(screen.getByRole('searchbox',{name:'Pesquisar função'}),{target:{value:'F5'}});
      fireEvent.change(screen.getByRole('combobox',{name:'Nova ação do botão selecionado'}),{target:{value:'key_f5'}});
      fireEvent.click(screen.getByRole('button',{name:'Aplicar botão selecionado'}));
      await waitFor(()=>expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(),'key_f5',`button_${n}`));
      await waitFor(()=>expect(screen.getByRole('button',{name:'Restaurar botão'}).hasAttribute('disabled')).toBe(false));
      fireEvent.click(screen.getByRole('button',{name:'Restaurar botão'}));
      await waitFor(()=>expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(),'restore_session',`button_${n}`));
      await waitFor(()=>expect(screen.getByRole('button',{name:'Restaurar botão'}).hasAttribute('disabled')).toBe(true));
    }
    expect(screen.queryByRole('textbox',{name:'Campo de teste de botão secundário'})).toBeNull();
  });
  it('only adds the second top rectangle and independently selects both top buttons without writes', async () => {
    const { container } = render(<X6Control onBack={() => undefined}/>);
    const svg = screen.getByRole('img', { name: /Ilustração esquemática original/ });
    expect(svg.getAttribute('viewBox')).toBe('0 0 320 500');
    expect(svg.querySelector('path')?.getAttribute('d')).toBe('M148 52 C112 52 66 70 62 103 C56 154 68 211 63 268 C55 324 47 369 72 416 C106 481 211 485 250 427 C280 382 264 323 258 264 C253 210 265 155 256 107 C250 78 207 56 177 52 Z');
    expect(svg.querySelector('rect[x="154"][y="203"][height="30"]')).toBeTruthy();
    expect(svg.querySelector('rect[x="154"][y="165"][height="30"]')).toBeTruthy();
    expect(svg.querySelector('rect[x="151"][y="99"][height="57"]')).toBeTruthy();
    expect(container.querySelectorAll('.x6-hotspot')).toHaveLength(7);
    await connect();
    const front=screen.getByRole('button',{name:'Selecionar Botão superior 1 (frente) no mouse'});
    const rear=screen.getByRole('button',{name:'Selecionar Botão superior 2 (trás) no mouse'});
    fireEvent.click(front);
    expect(screen.getByRole('button',{name:/6.*Botão superior 1/}).getAttribute('aria-pressed')).toBe('true');
    expect((screen.getByRole('combobox',{name:'Nova ação do botão selecionado'}) as HTMLSelectElement).value).toBe('dpi_up');
    expect((screen.getByRole('button',{name:'Aplicar botão selecionado'}) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByRole('button',{name:/7.*Botão superior 2/}));
    expect(rear.getAttribute('aria-pressed')).toBe('true');
    expect(front.getAttribute('aria-pressed')).toBe('false');
    expect((screen.getByRole('combobox',{name:'Nova ação do botão selecionado'}) as HTMLSelectElement).value).toBe('dpi_down');
    expect((screen.getByRole('button',{name:'Restaurar botão'}) as HTMLButtonElement).disabled).toBe(true);
    expect(starterClient.set).not.toHaveBeenCalled();
  });

  it('does not label an old six-button snapshot as a confirmed top mapping', async () => {
    const old=snapshot(); delete old.values.button_7;
    old.values.button_6='dpi_cycle'; old.settings=old.settings.filter((s)=>s.key!=='button_7');
    vi.mocked(starterClient.start).mockResolvedValue(old);
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.click(screen.getByRole('button',{name:'Selecionar Botão superior 1 (frente) no mouse'}));
    expect(screen.getAllByText('Snapshot antigo — atualize o plugin')).toHaveLength(2);
    expect((screen.getByRole('combobox',{name:'Nova ação do botão selecionado'}) as HTMLSelectElement).value).toBe('');
    expect((screen.getByRole('button',{name:'Aplicar botão selecionado'}) as HTMLButtonElement).disabled).toBe(true);
  });
  it('has three columns/tabs, no auto-connect, protected primary clicks and synchronized hotspots', async () => {
    render(<X6Control onBack={() => undefined}/>);
    expect(starterClient.start).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Conectar 2.4 GHz' }).hasAttribute('disabled')).toBe(false);
    expect(screen.queryByRole('checkbox', { name: /Confirmei X6/ })).toBeNull();
    expect(screen.queryByText(/Nenhum evento local|Somente neste campo|Teclas são posições HID|Um lateral por operação/)).toBeNull();
    expect((screen.getByRole('button', { name: 'Aplicar botão selecionado' }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getAllByRole('tab').map((e) => e.textContent)).toEqual(['Mouse', 'Macros', 'Diagnóstico', 'Iluminação']);
    await connect();
    expect(starterClient.start).toHaveBeenCalledWith('dev.attack-shark-x6', true);
    fireEvent.click(screen.getByRole('button', { name: 'Selecionar Lateral traseiro no mouse' }));
    expect(screen.getByRole('button', { name: /5.*Lateral traseiro/ }).getAttribute('aria-pressed')).toBe('true');
    expect((screen.getByRole('combobox', { name: 'Nova ação do botão selecionado' }) as HTMLSelectElement).value).toBe('key_3');
    fireEvent.click(screen.getByRole('button', { name: /1.*Clique esquerdo/ }));
    expect(screen.getByRole('button', { name: 'Selecionar Clique esquerdo no mouse' }).getAttribute('aria-pressed')).toBe('true');
    expect((screen.getByRole('combobox', { name: 'Nova ação do botão selecionado' }) as HTMLSelectElement).disabled).toBe(true);
    expect(starterClient.set).not.toHaveBeenCalled();
  });

  it('applies only the selected side button, does not claim success for draft and restores initial custom action', async () => {
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.change(screen.getByRole('combobox', { name: 'Nova ação do botão selecionado' }), { target: { value: 'key_a' } });
    expect(screen.queryByText(/Escrita confirmada por readback físico/)).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar botão selecionado' }));
    await screen.findAllByText(/Aplicar Lateral dianteiro · Escrita confirmada por readback físico/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.objectContaining({ hardwareAccess: true }), 'key_a', 'button_4');
    fireEvent.click(screen.getByRole('button', { name: 'Restaurar botão' }));
    await screen.findAllByText(/Restaurar Lateral dianteiro · Escrita confirmada por readback físico/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(), 'restore_session', 'button_4');
    expect((screen.getByRole('combobox', { name: 'Nova ação do botão selecionado' }) as HTMLSelectElement).value).toBe('key_2');
  });

  it('keeps DPI and polling on the existing physical RPC path, one setting at a time', async () => {
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.change(screen.getByRole('spinbutton', { name: 'DPI estágio 4' }), { target: { value: '3250' } });
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar DPI selecionado' }));
    await screen.findAllByText(/Aplicar DPI estágio 4 · Escrita confirmada/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(), 3250, 'dpi_stage_4');
    fireEvent.click(screen.getByRole('button', { name: '500 Hz' }));
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar polling' }));
    await screen.findAllByText(/Aplicar polling · Escrita confirmada/);
    expect(starterClient.set).toHaveBeenLastCalledWith(expect.anything(), 500, 'polling_rate_hz');
  });

  it('shows real diagnostics and explicitly unavailable macros without sending commands on tab navigation', async () => {
    render(<X6Control onBack={() => undefined}/>); await connect();
    fireEvent.click(screen.getByRole('tab', { name: 'Macros' }));
    expect(screen.getByText(/Nenhum report de macro é enviado/)).toBeTruthy();
    fireEvent.click(screen.getByRole('tab', { name: 'Diagnóstico' }));
    expect(screen.getByText('1d57:fa60')).toBeTruthy();
    expect(screen.getByText('hidraw_fixture')).toBeTruthy();
    expect(starterClient.set).not.toHaveBeenCalled();
    expect(starterClient.read).not.toHaveBeenCalled();
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Diagnóstico' }), { key: 'Home' });
    expect(screen.getByRole('tab', { name: 'Mouse' }).getAttribute('aria-selected')).toBe('true');
  });

  it('clears physical state on read/ACK errors and never fabricates button values or falls back', async () => {
    render(<X6Control onBack={() => undefined}/>); await connect();
    vi.mocked(starterClient.read).mockRejectedValue({ kind: 'ACK_FAILED', configurationOutcome: 'unknown' });
    fireEvent.click(screen.getByRole('button', { name: 'Ler botões' }));
    expect((await screen.findByRole('alert')).textContent).toContain('ACK ausente ou rejeitado');
    await waitFor(() => expect((screen.getByRole('button', { name: 'Aplicar botão selecionado' }) as HTMLButtonElement).disabled).toBe(true));
    expect(screen.getAllByText('Aguardando leitura…').length).toBeGreaterThan(0);
    expect(starterClient.start).toHaveBeenCalledTimes(1);
    expect(starterClient.stop).toHaveBeenCalled();
  });
});
